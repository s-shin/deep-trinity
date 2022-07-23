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

pub trait StackTreeNodeData<'a> {
    fn new(common_data: StackTreeCommonNodeData<'a>) -> Self;
    fn common_data(&self) -> &StackTreeCommonNodeData<'a>;
}

pub struct DefaultStackTreeNodeData<'a> {
    common_data: StackTreeCommonNodeData<'a>,
}

impl<'a> StackTreeNodeData<'a> for DefaultStackTreeNodeData<'a> {
    fn new(common_data: StackTreeCommonNodeData<'a>) -> Self { Self { common_data } }
    fn common_data(&self) -> &StackTreeCommonNodeData<'a> { &self.common_data }
}

pub type StackTreeNodeArena<NodeData> = VecNodeArena<NodeData>;

#[allow(unused_variables)]
pub trait StackTreeNodeExpansionFilter<'a> {
    type NodeData: StackTreeNodeData<'a>;
    /// Filter of the candidate's placement where the falling piece will be moved.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_destination(&mut self, node_data: &Self::NodeData, dst: &Placement) -> bool { true }
    /// Filter of the hold action.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_hold(&mut self, node_data: &Self::NodeData) -> bool { true }
    /// Filter to the game that will be contained in the new node data.
    /// At this time, the new node data is not created.
    fn filter_new_game(&mut self, node_data: &Self::NodeData, new_game: &Game) -> bool { true }
    /// Filter to the data of the new node.
    /// If false was returned, the data is discarded.
    fn filter_new_node_data(&mut self, node_data: &Self::NodeData, new_node_data: &Self::NodeData) -> bool { true }
}

#[derive(Default)]
pub struct DefaultStackTreeNodeExpansionFilter<'a, NodeData: StackTreeNodeData<'a>> {
    phantom: PhantomData<fn() -> &'a NodeData>,
}

impl<'a, NodeData: StackTreeNodeData<'a>> StackTreeNodeExpansionFilter<'a> for DefaultStackTreeNodeExpansionFilter<'a, NodeData> {
    type NodeData = NodeData;
}

pub struct StackTree<'a, NodeData: StackTreeNodeData<'a>> {
    arena: StackTreeNodeArena<NodeData>,
    root: NodeHandle,
    phantom: PhantomData<fn() -> &'a ()>,
}

impl<'a, NodeData: StackTreeNodeData<'a>> StackTree<'a, NodeData> {
    pub fn new(game: Game<'a>) -> Result<Self, &'static str> {
        let mut arena: StackTreeNodeArena<NodeData> = Default::default();
        let common_data = StackTreeCommonNodeData::new(None, game)?;
        let root = arena.create(NodeData::new(common_data));
        Ok(Self { arena, root, phantom: PhantomData })
    }
    pub fn arena(&self) -> &StackTreeNodeArena<NodeData> { &self.arena }
    pub fn arena_mut(&mut self) -> &mut StackTreeNodeArena<NodeData> { &mut self.arena }
    pub fn root(&self) -> NodeHandle { self.root }
    pub fn visit(&self, visitor: impl FnMut(&StackTreeNodeArena<NodeData>, NodeHandle, &mut VisitContext)) {
        self.arena.visit_depth_first(self.root, visitor);
    }
    pub fn expand(&mut self, target: NodeHandle, filter: &mut impl StackTreeNodeExpansionFilter<'a, NodeData=NodeData>) -> Result<(), &'static str> {
        let mut children_data = Vec::new();
        {
            let target_data = &self.arena[target].data;
            let common_data = target_data.common_data();

            if common_data.game.state.falling_piece.is_some() {
                for placement in common_data.move_decision_resource.dst_candidates.iter() {
                    if !filter.filter_destination(&target_data, placement) {
                        continue;
                    }
                    let mut game = common_data.game.clone();
                    game.state.falling_piece.as_mut().unwrap().placement = *placement;
                    if game.lock().unwrap() {
                        if !filter.filter_new_game(&target_data, &game) {
                            continue;
                        }
                        let by = Some(Action::Move(MoveTransition::new(*placement, None)));
                        let new_common_data = StackTreeCommonNodeData::new(by, game)?;
                        let new_data = NodeData::new(new_common_data);
                        if !filter.filter_new_node_data(&target_data, &new_data) {
                            continue;
                        }
                        children_data.push(new_data);
                    }
                }
            }

            // Using while for the readability.
            while common_data.game.state.can_hold {
                if !filter.filter_hold(&target_data) {
                    break;
                }
                let mut game = common_data.game.clone();
                game.hold().unwrap();
                if game.state.falling_piece.is_some() {
                    if !filter.filter_new_game(&target_data, &game) {
                        break;
                    }
                    let new_common_data = StackTreeCommonNodeData::new(Some(Action::Hold), game)?;
                    let new_data = NodeData::new(new_common_data);
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

pub trait Simulator<'a> {
    type NodeData: StackTreeNodeData<'a>;
    type NodeExpansionFilter: StackTreeNodeExpansionFilter<'a, NodeData=Self::NodeData>;
    fn select(&mut self, tree: &mut StackTree<'a, Self::NodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>>;
    fn expansion_filter(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle) -> Result<&mut Self::NodeExpansionFilter, Box<dyn Error>>;
    fn on_expanded(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle) -> Result<(), Box<dyn Error>>;
}

pub fn simulate_once<'a, NodeData, NodeExpansionFilter>(tree: &mut StackTree<'a, NodeData>, simulator: &mut impl Simulator<'a, NodeData=NodeData, NodeExpansionFilter=NodeExpansionFilter>) -> Result<bool, Box<dyn Error>> where
    NodeData: StackTreeNodeData<'a>,
    NodeExpansionFilter: StackTreeNodeExpansionFilter<'a, NodeData=NodeData>
{
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

    struct SimpleExpansionFilter {}

    impl<'a> StackTreeNodeExpansionFilter<'a> for SimpleExpansionFilter {
        type NodeData = DefaultStackTreeNodeData<'a>;
        fn filter_new_game(&mut self, _node_data: &Self::NodeData, new_game: &Game) -> bool {
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

    impl<'a> Simulator<'a> for SimpleSimulator {
        type NodeData = DefaultStackTreeNodeData<'a>;
        type NodeExpansionFilter = SimpleExpansionFilter;
        fn select(&mut self, _tree: &mut StackTree<'a, Self::NodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>> {
            let depth_first = true;
            let target = if depth_first {
                self.leaf_nodes.pop_back()
            } else {
                self.leaf_nodes.pop_front()
            };
            Ok(target)
        }
        fn expansion_filter(&mut self, _tree: &mut StackTree<'a, Self::NodeData>, _target: NodeHandle) -> Result<&mut Self::NodeExpansionFilter, Box<dyn Error>> {
            Ok(&mut self.filter)
        }
        fn on_expanded(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle) -> Result<(), Box<dyn Error>> {
            let children = tree.arena()[target].children();
            self.leaf_nodes.extend(children.iter());
            Ok(())
        }
    }

    #[test]
    fn test_stack_finder() {
        let now = Local::now().format("%Y%m%d_%H%M%S_%.3f").to_string();

        let enable_profiling = false;
        let profile_result_file_path = format!("tmp/{}-profile.pb", now);
        let max_expansion_count = 10;
        let enable_logging = true;
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

        let mut tree = StackTree::<DefaultStackTreeNodeData<'static>>::new(game).unwrap();
        let mut simulator = SimpleSimulator::new(tree.root());

        let mut last_i = 0;
        for i in 0.. {
            if i == max_expansion_count {
                last_i = i;
                break;
            }
            if i % progress_log_interval == 0 {
                writeln!(&mut log_file, "[{}] {}...", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), i).unwrap();
            }
            if !simulate_once(&mut tree, &mut simulator).unwrap() {
                last_i = i;
                break;
            }
        }
        writeln!(&mut log_file, "[{}] {} (finished)", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), last_i).unwrap();

        if enable_logging {
            tree.visit(|arena, node, ctx| {
                let n = &arena[node];
                let indent = "  ".repeat(ctx.depth());
                writeln!(&mut log_file, "{}- by_action: {:?}", indent, n.data.common_data().by).unwrap();
                writeln!(&mut log_file, "{}  game: |-\n{}", indent, n.data.common_data().game.to_string().split("\n")
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
