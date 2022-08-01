use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::marker::PhantomData;
use std::rc::Rc;
use std::str::FromStr;
use bitvec::prelude::*;
use core::{Piece, Placement, LineClear, Orientation, Game};
use tree::arena::{NodeArena, NodeHandle};
use crate::Action;
use crate::helper::stack_tree::{StackTree, StackTreeCommonNodeData, StackTreeNodeData, StackTreeNodeExpander, StackTreeSimulator};

//--------------------------------------------------------------------------------------------------
// PiecePlacement
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct PiecePlacement {
    pub piece: Piece,
    pub placement: Placement,
}

impl PiecePlacement {
    fn new(piece: Piece, placement: Placement) -> Self {
        Self { piece, placement }
    }
}

impl FromStr for PiecePlacement {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(",");
        let err_msg = "Invalid format.";
        let part0 = parts.next().ok_or::<Self::Err>(err_msg.into())?;
        let part1 = parts.next().ok_or::<Self::Err>(err_msg.into())?;
        let part2 = parts.next().ok_or::<Self::Err>(err_msg.into())?;
        let part3 = parts.next().ok_or::<Self::Err>(err_msg.into())?;

        let piece = if let Some(c) = part0.trim().chars().next() {
            if let Ok(p) = Piece::from_char(c) {
                p
            } else {
                return Err(format!("'{}' is not piece character.", c).into());
            }
        } else {
            return Err("A piece character is required..".into());
        };

        let orientation = u8::from_str(part1.trim())
            .map(Orientation::new)
            .map_err(|_| Self::Err::from("Invalid orientation value."))?;

        let x = i8::from_str(part2.trim()).map_err(|_| Self::Err::from("Invalid x value."))?;
        let y = i8::from_str(part3.trim()).map_err(|_| Self::Err::from("invalid y value."))?;

        Ok(Self::new(piece, Placement::new(orientation, (x, y).into())))
    }
}

//--------------------------------------------------------------------------------------------------
// NodeData
//--------------------------------------------------------------------------------------------------

struct NodeData<'a> {
    common_data: StackTreeCommonNodeData<'a>,
    patterns: Rc<Vec<Vec<PiecePlacement>>>,
    check_lists: Vec<BitVec>,
}

impl<'a> NodeData<'a> {
    fn new(common_data: StackTreeCommonNodeData<'a>, patterns: Rc<Vec<Vec<PiecePlacement>>>, check_lists: Vec<BitVec>) -> Self {
        Self { common_data, patterns, check_lists }
    }
    fn new_for_root(game: Game<'a>, patterns: Rc<Vec<Vec<PiecePlacement>>>) -> Result<Self, &'static str> {
        let common_data = StackTreeCommonNodeData::new(None, game)?;
        let check_lists = patterns.iter()
            .map(|pattern| bitvec!(0; pattern.len()))
            .collect::<Vec<_>>();
        Ok(Self { common_data, patterns, check_lists })
    }
}

impl<'a> StackTreeNodeData<'a> for NodeData<'a> {
    fn common_data(&self) -> &StackTreeCommonNodeData<'a> { &self.common_data }
}

//--------------------------------------------------------------------------------------------------
// NodeExpander
//--------------------------------------------------------------------------------------------------

#[derive(Default)]
struct NodeExpander {
    found: HashMap<Placement, (usize, usize)>,
}

impl<'a> StackTreeNodeExpander<'a> for NodeExpander {
    type NodeData = NodeData<'a>;

    fn filter_destination(&mut self, node_data: &Self::NodeData, dst: &Placement) -> bool {
        self.found.clear();
        let piece = node_data.common_data().game.state.falling_piece.as_ref().unwrap().piece();
        for (i, check_list) in node_data.check_lists.iter().enumerate() {
            let pattern = node_data.patterns.get(i).unwrap();
            let j = check_list.iter_zeros()
                .find(|&j| {
                    let pp = pattern.get(j).unwrap();
                    if pp.piece != piece {
                        return false;
                    }
                    if pp.placement != *dst {
                        return false;
                    }
                    true
                });
            if let Some(j) = j {
                self.found.insert(*dst, (i, j));
            }
        }
        !self.found.is_empty()
    }

    fn new_node_data(&mut self, node_data: &Self::NodeData, new_common_node_data: StackTreeCommonNodeData<'a>) -> Result<Option<Self::NodeData>, Box<dyn Error>> {
        let mut check_lists = node_data.check_lists.clone();
        if let Some(Action::Move(mt)) = new_common_node_data.by {
            let indices = self.found.get(&mt.placement).unwrap();
            check_lists.get_mut(indices.0).unwrap().set(indices.1, true);
        }
        Ok(Some(NodeData::new(new_common_node_data, node_data.patterns.clone(), check_lists)))
    }
}

//--------------------------------------------------------------------------------------------------
// Simulator
//--------------------------------------------------------------------------------------------------

#[derive(Debug)]
struct Simulator {
    leaf_nodes: VecDeque<NodeHandle>,
    found: Vec<NodeHandle>,
}

impl Simulator {
    pub fn new(root: NodeHandle) -> Self {
        let leaf_nodes = VecDeque::from([root]);
        Self { leaf_nodes, found: vec![] }
    }
}

impl<'a> StackTreeSimulator<'a> for Simulator {
    type NodeData = NodeData<'a>;
    type NodeExpander = NodeExpander;

    fn select(&mut self, tree: &mut StackTree<'a, Self::NodeData>) -> Result<Option<NodeHandle>, Box<dyn Error>> {
        Ok(self.leaf_nodes.pop_back())
    }

    fn expander(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle) -> Result<Self::NodeExpander, Box<dyn Error>> {
        Ok(NodeExpander::default())
    }

    fn on_expanded(&mut self, tree: &mut StackTree<'a, Self::NodeData>, target: NodeHandle, expander: &Self::NodeExpander) -> Result<(), Box<dyn Error>> {
        self.leaf_nodes.extend(tree.arena().get(target).unwrap().children());
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use rand::prelude::*;
    use crate::helper::stack_tree::simulate_once;
    use crate::RandomPieceGenerator;
    use super::*;

    #[test]
    fn test_piece_placement_from_str() {
        assert_eq!(
            PiecePlacement::new(Piece::I, Placement::new(Orientation::new(0), (1, 2).into())),
            PiecePlacement::from_str("I,0,1,2").unwrap(),
        );
    }

    #[test]
    fn test_simulator() {
        // TODO
        let mut patterns = Rc::new(vec![vec![]]);

        let mut game: Game<'static> = Default::default();
        let mut rpg = RandomPieceGenerator::new(StdRng::seed_from_u64(0));
        game.supply_next_pieces(&rpg.generate());
        game.supply_next_pieces(&rpg.generate());
        game.supply_next_pieces(&rpg.generate());
        game.setup_falling_piece(None).unwrap();

        let mut tree = StackTree::new(NodeData::new_for_root(game, patterns.clone()).unwrap()).unwrap();
        let mut simulator = Simulator::new(tree.root());
        for _i in 0..5000 {
            if !simulate_once(&mut tree, &mut simulator).unwrap() {
                break;
            }
        }

        println!("{:?}", &simulator);
    }
}
