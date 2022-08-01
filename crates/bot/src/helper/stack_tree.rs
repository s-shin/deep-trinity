use std::error::Error;
use std::io::Write;
use std::marker::PhantomData;
use core::{Game, Placement};
use core::helper::MoveDecisionResource;
use tree::arena::{NodeArena, NodeHandle, VecNodeArena, VisitContext};
use crate::{Action, MoveTransition};

//--------------------------------------------------------------------------------------------------
// StackTreeNodeData
//--------------------------------------------------------------------------------------------------

pub struct StackTreeCommonNodeData<'a> {
    pub by: Option<Action>,
    pub game: Game<'a>,
    pub move_decision_resource: MoveDecisionResource,
}

impl<'a> StackTreeCommonNodeData<'a> {
    pub fn new(by: Option<Action>, game: Game<'a>) -> Result<Self, &'static str> {
        let move_decision_resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by, game, move_decision_resource })
    }
}

pub trait StackTreeNodeData<'a> {
    fn common_data(&self) -> &StackTreeCommonNodeData<'a>;
}

pub struct DefaultStackTreeNodeData<'a> {
    common_data: StackTreeCommonNodeData<'a>,
}

impl<'a> DefaultStackTreeNodeData<'a> {
    fn new(common_data: StackTreeCommonNodeData<'a>) -> Self { Self { common_data } }
}

impl<'a> StackTreeNodeData<'a> for DefaultStackTreeNodeData<'a> {
    fn common_data(&self) -> &StackTreeCommonNodeData<'a> { &self.common_data }
}

//--------------------------------------------------------------------------------------------------
// StackTreeNodeExpander
//--------------------------------------------------------------------------------------------------

/// Trait of the implementation that is passed to [StackTree::expand].
/// For each call of [StackTree::expand], one instance will be created
/// by [StackTreeSimulator::expander] (also see [simulate_once]).
/// In this trait, there are some filter methods due to get better performance.
/// The trait methods can be called sequentially by two patterns;
/// in moving a piece to a destination,
/// `filter_destination` -> `filter_new_game` -> `new_node_data`,
/// and in holding a piece,
/// `filter_hold` -> `filter_new_game` -> `new_node_data`.
pub trait StackTreeNodeExpander<'a> {
    type NodeData: StackTreeNodeData<'a>;
    /// Filter of the candidate's placement where the falling piece will be moved.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_destination(&mut self, _node_data: &Self::NodeData, _dst: &Placement) -> bool { true }
    /// Filter of the hold action.
    /// At this time, the game in `node_data` is not cloned.
    fn filter_hold(&mut self, _node_data: &Self::NodeData) -> bool { true }
    /// Filter to the game that will be contained in the new node data.
    /// At this time, the new common node data is not created.
    fn filter_new_game(&mut self, _node_data: &Self::NodeData, _new_game: &Game) -> bool { true }
    /// Factory method to create a new node data with `new_common_node_data`.
    /// If `Ok(None)` was returned, `new_common_node_data` is discarded and no new node is appended to the tree.
    fn new_node_data(&mut self, node_data: &Self::NodeData, new_common_node_data: StackTreeCommonNodeData<'a>) -> Result<Option<Self::NodeData>, Box<dyn Error>>;
}

//--------------------------------------------------------------------------------------------------
// StackTree
//--------------------------------------------------------------------------------------------------

pub type StackTreeNodeArena<NodeData> = VecNodeArena<NodeData>;

pub struct StackTree<'a, NodeData: StackTreeNodeData<'a>> {
    arena: StackTreeNodeArena<NodeData>,
    root: NodeHandle,
    phantom: PhantomData<fn() -> &'a ()>,
}

impl<'a, NodeData: StackTreeNodeData<'a>> StackTree<'a, NodeData> {
    pub fn new(root_node_data: NodeData) -> Result<Self, &'static str> {
        let mut arena: StackTreeNodeArena<NodeData> = Default::default();
        let root = arena.create(root_node_data);
        Ok(Self { arena, root, phantom: PhantomData })
    }
    pub fn arena(&self) -> &StackTreeNodeArena<NodeData> { &self.arena }
    pub fn arena_mut(&mut self) -> &mut StackTreeNodeArena<NodeData> { &mut self.arena }
    pub fn root(&self) -> NodeHandle { self.root }
    pub fn visit(&self, visitor: impl FnMut(&StackTreeNodeArena<NodeData>, NodeHandle, &mut VisitContext)) {
        self.arena.visit_depth_first(self.root, visitor);
    }
    pub fn expand(&mut self, target: NodeHandle, expander: &mut impl StackTreeNodeExpander<'a, NodeData=NodeData>) -> Result<(), Box<dyn Error>> {
        let mut children_data = Vec::new();
        {
            let target_data = &self.arena[target].data;
            let common_data = target_data.common_data();

            if common_data.game.state.falling_piece.is_some() {
                for placement in common_data.move_decision_resource.dst_candidates.iter() {
                    if !expander.filter_destination(&target_data, placement) {
                        continue;
                    }
                    let mut game = common_data.game.clone();
                    game.state.falling_piece.as_mut().unwrap().placement = *placement;
                    if game.lock().unwrap() {
                        if !expander.filter_new_game(&target_data, &game) {
                            continue;
                        }
                        let by = Some(Action::Move(MoveTransition::new(*placement, None)));
                        let new_common_data = StackTreeCommonNodeData::new(by, game)?;
                        if let Some(new_data) = expander.new_node_data(&target_data, new_common_data)? {
                            children_data.push(new_data);
                        }
                    }
                }
            }

            // Using only one loop while for the readability.
            while common_data.game.state.can_hold {
                if !expander.filter_hold(&target_data) {
                    break;
                }
                let mut game = common_data.game.clone();
                game.hold().unwrap();
                if game.state.falling_piece.is_some() {
                    if !expander.filter_new_game(&target_data, &game) {
                        break;
                    }
                    let new_common_data = StackTreeCommonNodeData::new(Some(Action::Hold), game)?;
                    if let Some(new_data) = expander.new_node_data(&target_data, new_common_data)? {
                        children_data.push(new_data);
                    }
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

//--------------------------------------------------------------------------------------------------
// StackTreeSimulator
//--------------------------------------------------------------------------------------------------

pub trait StackTreeSimulator<'a> {
    type NodeData: StackTreeNodeData<'a>;
    type NodeExpander: StackTreeNodeExpander<'a, NodeData=Self::NodeData>;
    fn select(&mut self, tree: &mut StackTree<'a, Self::NodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>>;
    fn expander(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle) -> Result<Self::NodeExpander, Box<dyn Error>>;
    fn on_expanded(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle, expander: &Self::NodeExpander) -> Result<(), Box<dyn Error>>;
}

pub fn simulate_once<'a, NodeData, NodeExpander>(
    tree: &mut StackTree<'a, NodeData>,
    simulator: &mut impl StackTreeSimulator<'a, NodeData=NodeData, NodeExpander=NodeExpander>,
) -> Result<bool, Box<dyn Error>> where
    NodeData: StackTreeNodeData<'a>,
    NodeExpander: StackTreeNodeExpander<'a, NodeData=NodeData>
{
    if let Some(target) = simulator.select(tree)? {
        let mut expander = simulator.expander(tree, target)?;
        tree.expand(target, &mut expander)?;
        simulator.on_expanded(tree, target, &expander)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::RandomPieceGenerator;
    use super::*;
    use std::collections::VecDeque;
    use std::fmt;
    use std::fmt::Formatter;
    use std::fs::File;
    use std::time::SystemTime;
    use rand::thread_rng;
    use chrono::prelude::*;
    use prost::Message;
    use grid::Grid;

    #[derive(Default)]
    struct ExpansionStats {
        new_games: i32,
        max_height_violations: i32,
        spaces_violations: i32,
    }

    impl ExpansionStats {
        fn merge(&mut self, other: &Self) {
            self.new_games += other.new_games;
            self.max_height_violations += other.max_height_violations;
            self.spaces_violations += other.spaces_violations;
        }
    }

    impl fmt::Display for ExpansionStats {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "new_games: {}, max_height_violations: {}, spaces_violations: {}",
                   self.new_games, self.max_height_violations, self.spaces_violations)
        }
    }

    #[derive(Default)]
    struct SimpleExpander {
        stats: ExpansionStats,
    }

    impl<'a> StackTreeNodeExpander<'a> for SimpleExpander {
        type NodeData = DefaultStackTreeNodeData<'a>;

        fn filter_new_game(&mut self, _node_data: &Self::NodeData, new_game: &Game) -> bool {
            self.stats.new_games += 1;

            let pf = &new_game.state.playfield;
            let max_height = 4;
            if pf.stack_height() > max_height {
                self.stats.max_height_violations += 1;
                return false;
            }

            // Each count of separated spaces should be a multiple of 4.
            let spaces = pf.grid.search_spaces((0, 0).into(), (pf.width(), max_height).into());
            for space in spaces.iter() {
                if space.len() % 4 != 0 {
                    self.stats.spaces_violations += 1;
                    return false;
                }
            }

            true
        }

        fn new_node_data(&mut self, _node_data: &Self::NodeData, new_common_node_data: StackTreeCommonNodeData<'a>) -> Result<Option<Self::NodeData>, Box<dyn Error>> {
            Ok(Some(Self::NodeData::new(new_common_node_data)))
        }
    }

    const DEPTH_FIRST: bool = true;

    struct SimpleSimulator {
        leaf_nodes: VecDeque<NodeHandle>,
        stats: ExpansionStats,
    }

    impl SimpleSimulator {
        fn new(target: NodeHandle) -> Self {
            let leaf_nodes = VecDeque::from([target]);
            Self { leaf_nodes, stats: Default::default() }
        }
        fn next_node(&self) -> Option<NodeHandle> {
            if DEPTH_FIRST {
                self.leaf_nodes.back().copied()
            } else {
                self.leaf_nodes.front().copied()
            }
        }
        fn pop_next_node(&mut self) -> Option<NodeHandle> {
            if DEPTH_FIRST {
                self.leaf_nodes.pop_back()
            } else {
                self.leaf_nodes.pop_front()
            }
        }
        fn info<'a>(&self, tree: &StackTree<'a, DefaultStackTreeNodeData<'a>>) -> String {
            let route = self.next_node()
                .map(|n| {
                    tree.arena().route(n).iter()
                        .map(|&n| format!("{}", n))
                        .collect::<Vec<_>>()
                        .join(" -> ")
                })
                .unwrap_or("end".into());
            format!("leaf_node_count: {}, expansion_stats: {{ {} }}, next: {}",
                    self.leaf_nodes.len(), self.stats, route)
        }
    }

    impl<'a, 'b> StackTreeSimulator<'a> for SimpleSimulator {
        type NodeData = DefaultStackTreeNodeData<'a>;
        type NodeExpander = SimpleExpander;
        fn select(&mut self, _tree: &mut StackTree<'a, Self::NodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>> {
            Ok(self.pop_next_node())
        }
        fn expander(&mut self, _tree: &mut StackTree<'a, Self::NodeData>, _target: NodeHandle) -> Result<Self::NodeExpander, Box<dyn Error>> {
            Ok(Self::NodeExpander::default())
        }
        fn on_expanded(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle, expander: &Self::NodeExpander) -> Result<(), Box<dyn Error>> {
            let children = tree.arena()[target].children();
            self.leaf_nodes.extend(children.iter());
            self.stats.merge(&expander.stats);
            Ok(())
        }
    }

    #[test]
    #[ignore]
    fn test_stack_finder() {
        let now = Local::now().format("%Y%m%d_%H%M%S_%.3f").to_string();

        enum LogSink {
            Null,
            Stdio,
            Stderr,
            File(String),
        }

        let enable_profiling = false;
        let profile_result_file_path = format!("tmp/{}-profile.pb", now);
        // let max_expansion_count = 10;
        let max_expansion_count = -1;
        let enable_logging = true;
        // let log_file_path = LogSink::File(format!("tmp/{}.log", now));
        let log_sink = LogSink::Stderr;
        let progress_log_interval = 1000;
        let enable_debug_log = false;
        let performance_mode = true;

        let guard = if enable_profiling {
            let guard = pprof::ProfilerGuardBuilder::default()
                .frequency(1000).blocklist(&["libc", "libgcc", "pthread", "vdso"]).build().unwrap();
            Some(guard)
        } else {
            None
        };

        let mut log_file: Box<dyn Write> = match log_sink {
            LogSink::Null => Box::new(std::io::sink()),
            LogSink::Stdio => Box::new(std::io::stdout()),
            LogSink::Stderr => Box::new(std::io::stderr()),
            LogSink::File(path) => Box::new(File::create(path).unwrap())
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

        let root_node_data = DefaultStackTreeNodeData::new(StackTreeCommonNodeData::new(None, game).unwrap());
        let mut tree = StackTree::<DefaultStackTreeNodeData<'static>>::new(root_node_data).unwrap();
        let mut simulator = SimpleSimulator::new(tree.root());

        let mut last_i = 0;
        for i in 0.. {
            if i == max_expansion_count {
                last_i = i;
                break;
            }
            if i % progress_log_interval == 0 {
                writeln!(&mut log_file, "[{}] {}... ({})", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), i, simulator.info(&tree)).unwrap();
            }
            if !simulate_once(&mut tree, &mut simulator).unwrap() {
                last_i = i;
                break;
            }
        }
        writeln!(&mut log_file, "[{}] {} (finished)", Local::now().to_rfc3339_opts(SecondsFormat::Millis, false), last_i).unwrap();

        let mut found = Vec::new();
        let root_pc_count = tree.arena()[tree.root()].data.common_data().game.stats.perfect_clear;
        tree.visit(|tree, node, _| {
            if tree[node].data.common_data.game.stats.perfect_clear > root_pc_count {
                found.push(node);
            }
        });
        writeln!(&mut log_file, "found: {:?}", found).unwrap();

        if enable_logging && enable_debug_log {
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
