use std::error::Error;
use std::io::Write;
use std::marker::PhantomData;
use core::{Game, Placement};
use core::helper::MoveDecisionResource;
use tree::arena::{Node, NodeArena, NodeHandle, VecNodeArena, VisitContext};
use crate::{Action, MoveTransition};

pub struct StackTreeCommonNodeData<'a> {
    by: Option<Action>,
    game: Game<'a>,
    move_decision_resource: MoveDecisionResource,
}

impl<'a> StackTreeCommonNodeData<'a> {
    fn new(by: Option<Action>, game: Game<'a>) -> Result<Self, &'static str> {
        let move_decision_resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by, game, move_decision_resource })
    }
}

pub trait StackTreeNodeDataTrait<'a> {
    fn new(common_data: StackTreeCommonNodeData) -> Self;
    fn common_data(&self) -> &StackTreeCommonNodeData<'a>;
}

pub struct DefaultStackTreeNodeData<'a> {
    common_data: StackTreeCommonNodeData<'a>,
}

impl<'a> StackTreeNodeDataTrait<'a> for DefaultStackTreeNodeData<'a> {
    fn new(common_data: StackTreeCommonNodeData) -> Self { Self { common_data } }
    fn common_data(&self) -> &StackTreeCommonNodeData<'a> { &self.common_data }
}

pub struct StackTreeNodeData<'a, ExtraNodeData> {
    by: Option<Action>,
    game: Game<'a>,
    move_decision_resource: MoveDecisionResource,
    extra: ExtraNodeData,
}

impl<'a, ExtraNodeData> StackTreeNodeData<'a, ExtraNodeData> {
    pub fn new(by: Option<Action>, game: Game<'a>, extra: ExtraNodeData) -> Result<Self, &'static str> {
        let move_decision_resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by, game, move_decision_resource, extra })
    }
}

pub type StackTreeNodeArena<'a, ExtraNodeData> = VecNodeArena<StackTreeNodeData<'a, ExtraNodeData>>;

#[allow(unused_variables)]
pub trait StackTreeNodeExpansionFilter {
    type ExtraNodeData;
    /// Filter of the candidate's placement where the falling piece will be moved.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_destination(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>, dst: &Placement) -> bool { true }
    /// Filter of the hold action.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_hold(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>) -> bool { true }
    /// Filter to the game that will be contained in the new node data.
    /// At this time, the new node data is not created.
    fn filter_new_game(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>, new_game: &Game) -> bool { true }
    /// Filter to the data of the new node.
    /// If false was returned, the data is discarded.
    fn filter_new_node_data(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>, new_node_data: &StackTreeNodeData<Self::ExtraNodeData>) -> bool { true }
    fn create_extra_data(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>, new_game: )
}

#[derive(Default)]
pub struct DefaultStackTreeNodeExpansionFilter<ExtraNodeData> {
    phantom: PhantomData<fn() -> ExtraNodeData>,
}

impl<ExtraNodeData> StackTreeNodeExpansionFilter for DefaultStackTreeNodeExpansionFilter<ExtraNodeData> {
    type ExtraNodeData = ExtraNodeData;
}

pub struct StackTree<'a, ExtraNodeData> {
    arena: StackTreeNodeArena<'a, ExtraNodeData>,
    root: NodeHandle,
}

impl<'a, ExtraNodeData> StackTree<'a, ExtraNodeData> {
    pub fn new(game: Game<'a>, extra: ExtraNodeData) -> Result<Self, &'static str> {
        let mut arena: StackTreeNodeArena<'a, ExtraNodeData> = Default::default();
        let root = arena.create(StackTreeNodeData::new(None, game, extra)?);
        Ok(Self { arena, root })
    }
    pub fn arena(&self) -> &StackTreeNodeArena<'a, ExtraNodeData> { &self.arena }
    pub fn arena_mut(&mut self) -> &mut StackTreeNodeArena<'a, ExtraNodeData> { &mut self.arena }
    pub fn root(&self) -> NodeHandle { self.root }
    pub fn visit(&self, visitor: impl FnMut(&StackTreeNodeArena<'a, ExtraNodeData>, NodeHandle, &mut VisitContext)) {
        self.arena.visit_depth_first(self.root, visitor);
    }
    pub fn expand(&mut self, target: NodeHandle, filter: &mut impl StackTreeNodeExpansionFilter<ExtraNodeData=ExtraNodeData>) -> Result<(), &'static str> {
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

pub trait Simulator {
    type ExtraNodeData;
    fn select(&mut self, tree: &mut StackTree<Self::ExtraNodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>>;
    fn expansion_filter<Filter: StackTreeNodeExpansionFilter<ExtraNodeData=Self::ExtraNodeData>>(&mut self, tree: &mut StackTree<Self::ExtraNodeData>, target: NodeHandle) -> Result<&mut Filter, Box<dyn Error>>;
    fn on_expanded(&mut self, tree: &mut StackTree<Self::ExtraNodeData>, target: NodeHandle) -> Result<(), Box<dyn Error>>;
}

pub fn simulate_once<ExtraNodeData>(tree: &mut StackTree<ExtraNodeData>, simulator: &mut impl Simulator<ExtraNodeData=ExtraNodeData>) -> Result<bool, Box<dyn Error>> {
    if let Some(target) = simulator.select(tree)? {
        let filter = simulator.expansion_filter(tree, target)?;
        tree.expand(target, filter)?;
        simulator.on_expanded(tree, target)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use crate::RandomPieceGenerator;
    use super::*;
    use std::collections::VecDeque;
    use std::fs::File;
    use std::time::SystemTime;
    use rand::thread_rng;
    use chrono::prelude::*;
    use prost::Message;
    use grid::Grid;

    struct SimpleExpansionFilter {}

    impl StackTreeNodeExpansionFilter for SimpleExpansionFilter {
        type ExtraNodeData = ();
        fn filter_new_game(&mut self, node_data: &StackTreeNodeData<Self::ExtraNodeData>, new_game: &Game) -> bool {
            let max_height = 4;
            new_game.state.playfield.stack_height() <= max_height
        }
    }

    struct SimpleSimulator {
        leaf_nodes: VecDeque<NodeHandle>,
        filter: SimpleExpansionFilter,
    }

    impl SimpleSimulator {
        fn new(target: NodeHandle) -> Self {
            let leaf_nodes = VecDeque::from([target]);
            let filter = SimpleExpansionFilter {};
            Self { leaf_nodes, filter }
        }
    }

    impl Simulator for SimpleSimulator {
        type ExtraNodeData = ();
        fn select(&mut self, tree: &mut StackTree<Self::ExtraNodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>> {
            let depth_first = true;
            let target = if depth_first {
                self.leaf_nodes.pop_back()
            } else {
                self.leaf_nodes.pop_front()
            };
            Ok(target)
        }
        fn expansion_filter<Filter: StackTreeNodeExpansionFilter<ExtraNodeData=Self::ExtraNodeData>>(&mut self, tree: &mut StackTree<Self::ExtraNodeData>, target: NodeHandle) -> Result<&mut Filter, Box<dyn Error>> {
            Ok(&mut self.filter)
        }
        fn on_expanded(&mut self, tree: &mut StackTree<Self::ExtraNodeData>, target: NodeHandle) -> Result<(), Box<dyn Error>> {
            let children = tree.arena()[target].children();
            self.leaf_nodes.extend(&children);
            Ok(())
        }
    }

    #[test]
    fn test_stack_finder() {
        let now = Local::now().format("%Y%m%d_%H%M%S_%.3f").to_string();

        let enable_profiling = false;
        let profile_result_file_path = format!("tmp/{}-profile.pb", now);
        let max_expansion_count = 10;
        let enable_logging = false;
        let log_file_path = format!("tmp/{}.log", now);
        let progress_log_interval = 10;
        let performance_mode = false;

        let guard = if enable_profiling {
            let guard = pprof::ProfilerGuardBuilder::default()
                .frequency(1000).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();
            Some(guard)
        } else {
            None
        };

        let mut log_file: Box<dyn Write> = if enable_logging {
            Box::new(File::create(log_file_path).unwrap())
        } else {
            Box::new(std::io::sink())
        };

        let mut game: Game<'static> = Default::default();
        if performance_mode {
            game.performance_mode();
        }
        let mut rpg = RandomPieceGenerator::new(thread_rng());
        for _ in 0..3 {
            game.supply_next_pieces(&rpg.generate());
        }
        game.setup_falling_piece(None).unwrap();

        let mut tree = StackTree::new(game, ()).unwrap();
        let mut simulator = SimpleSimulator::new(tree.root());

        // let mut leaf_nodes = VecDeque::from([tree.root()]);
        // let depth_first = false;
        // let max_height = 2;
        let mut i;
        for i in 0.. {
            if i == max_expansion_count {
                break;
            }
            if i % progress_log_interval == 0 {
                writeln!(&mut log_file, "[{}] {}...", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), i).unwrap();
            }
            if !simulate_once(&mut simulator)? {
                break;
            }

            // let target = if depth_first { leaf_nodes.pop_back() } else { leaf_nodes.pop_front() };
            // let target = match target {
            //     Some(h) => h,
            //     _ => break,
            // };
            //
            // tree.expand(target, &mut DefaultStackTreeNodeExpansionFilter::default()).unwrap();
            //
            // let children = tree.arena()[target].children().iter()
            //     .filter(|&&h| {
            //         let grid = &tree.arena()[h].data.game.state.playfield.grid;
            //         grid.height() - grid.top_padding() <= max_height
            //     });
            // leaf_nodes.extend(children);
        }
        writeln!(&mut log_file, "[{}] ...{}", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), i).unwrap();

        if enable_logging {
            tree.visit(|arena, node, ctx| {
                let n = &arena[node];
                let indent = "  ".repeat(ctx.depth());
                writeln!(&mut log_file, "{}- by_action: {:?}", indent, n.data.by).unwrap();
                writeln!(&mut log_file, "{}  game: |-\n{}", indent, n.data.game.to_string().split("\n")
                    .map(|line| format!("{}    {}", indent, line)).collect::<Vec<_>>().join("\n")).unwrap();
                writeln!(&mut log_file, "{}  children: {}", indent, if n.is_leaf() { "[]" } else { "" }).unwrap();
            });
        }

        if let Some(guard) = guard {
            if let Ok(report) = guard.report().build() {
                let mut file = File::create(profile_result_file_path).unwrap();
                let profile = report.pprof().unwrap();
                let mut content = Vec::new();
                profile.encode(&mut content).unwrap();
                file.write_all(&content).unwrap();
            }
        }
    }
}
