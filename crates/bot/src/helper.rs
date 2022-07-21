use std::io::Write;
use core::{Game, Placement};
use core::helper::MoveDecisionResource;
use tree::arena::{Node, NodeArena, NodeHandle, VecNodeArena, VisitContext};
use crate::{Action, MoveTransition};

pub struct StackTreeNodeData<'a> {
    by: Option<Action>,
    game: Game<'a>,
    move_decision_resource: MoveDecisionResource,
}

impl<'a> StackTreeNodeData<'a> {
    pub fn new(by: Option<Action>, game: Game<'a>) -> Result<Self, &'static str> {
        let move_decision_resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by, game, move_decision_resource })
    }
}

pub type StackTreeNodeArena<'a> = VecNodeArena<StackTreeNodeData<'a>>;

#[allow(unused_variables)]
pub trait StackTreeNodeExpansionFilter {
    /// Filter of the candidate's placement where the falling piece will be moved.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_destination(&mut self, node_data: &StackTreeNodeData, dst: &Placement) -> bool { true }
    /// Filter of the hold action.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_hold(&mut self, node_data: &StackTreeNodeData) -> bool { true }
    /// Filter to the game that will be contained in the new node data.
    /// At this time, the new node data is not created.
    fn filter_new_game(&mut self, node_data: &StackTreeNodeData, new_game: &Game) -> bool { true }
    /// Filter to the data of the new node.
    /// If false was returned, the data is discarded.
    fn filter_new_node_data(&mut self, node_data: &StackTreeNodeData, new_node_data: &StackTreeNodeData) -> bool { true }
}

#[derive(Default)]
pub struct DefaultStackTreeNodeExpansionFilter;

impl StackTreeNodeExpansionFilter for DefaultStackTreeNodeExpansionFilter {}

pub struct StackTree<'a> {
    arena: StackTreeNodeArena<'a>,
    root: NodeHandle,
}

impl<'a> StackTree<'a> {
    pub fn new(game: Game<'a>) -> Result<Self, &'static str> {
        let mut arena: StackTreeNodeArena<'a> = Default::default();
        let root = arena.create(StackTreeNodeData::new(None, game)?);
        Ok(Self { arena, root })
    }
    pub fn arena(&self) -> &StackTreeNodeArena<'a> { &self.arena }
    pub fn arena_mut(&mut self) -> &mut StackTreeNodeArena<'a> { &mut self.arena }
    pub fn root(&self) -> NodeHandle { self.root }
    pub fn visit(&self, visitor: impl FnMut(&StackTreeNodeArena, NodeHandle, &mut VisitContext)) {
        self.arena.visit_depth_first(self.root, visitor);
    }
    pub fn expand(&mut self, target: NodeHandle, filter: &mut impl StackTreeNodeExpansionFilter) -> Result<(), &'static str> {
        let mut children_data = Vec::new();
        {
            let target_data = &self.arena[target].data;

            if target_data.game.state.falling_piece.is_some() {
                for placement in target_data.move_decision_resource.dst_candidates.iter() {
                    if !filter.filter_destination(&target_data, placement) {
                        continue;
                    }
                    let mut game = target_data.game.clone();
                    game.state.falling_piece.as_mut().unwrap().placement = *placement;
                    if game.lock().unwrap() {
                        if !filter.filter_new_game(&target_data, &game) {
                            continue;
                        }
                        let by = Some(Action::Move(MoveTransition::new(*placement, None)));
                        let new_data = StackTreeNodeData::new(by, game)?;
                        if !filter.filter_new_node_data(&target_data, &new_data) {
                            continue;
                        }
                        children_data.push(new_data);
                    }
                }
            }

            // Using while for the readability.
            while target_data.game.state.can_hold {
                if !filter.filter_hold(&target_data) {
                    break;
                }
                let mut game = target_data.game.clone();
                game.hold().unwrap();
                if game.state.falling_piece.is_some() {
                    if !filter.filter_new_game(&target_data, &game) {
                        break;
                    }
                    let new_data = StackTreeNodeData::new(Some(Action::Hold), game)?;
                    if !filter.filter_new_node_data(&target_data, &new_data) {
                        break;
                    }
                    children_data.push(new_data);
                }
                break;
            }
        }

        while let Some(data) = children_data.pop() {
            self.arena.append_child(target, data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rand::thread_rng;
    use crate::RandomPieceGenerator;
    use super::*;
    use std::collections::VecDeque;
    use std::fs::File;
    use prost::Message;
    use grid::Grid;

    #[test]
    fn test_stack_finder() {
        // let guard = pprof::ProfilerGuardBuilder::default()
        //     .frequency(1000).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();

        // let cwd = std::env::current_dir().unwrap();
        // println!("{}", cwd.display());
        // return;
        let mut log_file = File::create("tmp/a.log").unwrap();

        let mut game: Game<'static> = Default::default();
        // game.fast_mode();
        let mut rpg = RandomPieceGenerator::new(thread_rng());
        game.supply_next_pieces(&rpg.generate());
        game.supply_next_pieces(&rpg.generate());
        game.setup_falling_piece(None).unwrap();

        let mut tree = StackTree::new(game).unwrap();
        let mut leaf_nodes = VecDeque::from([tree.root()]);
        let depth_first = false;
        let max_height = 2;
        const N: i32 = 10;
        for i in 0.. {
            if i % 10 == 0 {
                writeln!(&mut log_file, "{}...", i).unwrap();
            }
            if i == N {
                break;
            }

            let target = if depth_first { leaf_nodes.pop_back() } else { leaf_nodes.pop_front() };
            let target = match target {
                Some(h) => h,
                _ => break,
            };

            tree.expand(target, &mut DefaultStackTreeNodeExpansionFilter::default()).unwrap();

            let children = tree.arena()[target].children().iter()
                .filter(|&&h| {
                    let grid = &tree.arena()[h].data.game.state.playfield.grid;
                    grid.height() - grid.top_padding() <= max_height
                });
            leaf_nodes.extend(children);
        }

        tree.visit(|arena, node, ctx| {
            let n = &arena[node];
            let indent = "  ".repeat(ctx.depth());
            writeln!(&mut log_file, "{}- by_action: {:?}", indent, n.data.by).unwrap();
            writeln!(&mut log_file, "{}  game: |-\n{}", indent, n.data.game.to_string().split("\n")
                .map(|line| format!("{}    {}", indent, line)).collect::<Vec<_>>().join("\n")).unwrap();
            writeln!(&mut log_file, "{}  children: {}", indent, if n.is_leaf() { "[]" } else { "" }).unwrap();
        });

        // if let Ok(report) = guard.report().build() {
        //     let mut file = File::create("tmp/profile.pb").unwrap();
        //     let profile = report.pprof().unwrap();
        //     let mut content = Vec::new();
        //     profile.encode(&mut content).unwrap();
        //     file.write_all(&content).unwrap();
        // }
    }
}
