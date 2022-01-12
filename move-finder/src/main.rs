use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;
use core::{Piece, Placement, MoveTransition, FallingPiece};
use core::helper::{MoveDecisionHelper, MoveDecisionStuff};
use bot::Action;
use tree::VisitPlan;

type Game = core::Game<'static>;

struct NodeData {
    by_action: Option<Action>,
    game: Game,
    pps: Vec<Rc<PiecePlacement>>,
    stuff: Rc<MoveDecisionStuff>,
}

impl NodeData {
    pub fn new(by_action: Option<Action>, game: Game, pps: Vec<Rc<PiecePlacement>>) -> Result<Self, &'static str> {
        let stuff = game.get_move_decision_helper(None)?.stuff;
        Ok(Self { by_action, game, pps, stuff })
    }
}

fn expand_node(node: &mut Rc<RefCell<tree::Node<NodeData>>>) -> Result<bool, &'static str> {
    let fp = node.borrow().data.game.state.falling_piece.clone();
    if let Some(fp) = fp {
        let pps_len = node.borrow().data.pps.len();
        for i in 0..pps_len {
            let pp = node.borrow().data.pps.get(i).cloned().unwrap();
            let (game, pps) = {
                let data = &node.borrow().data;
                if pp.piece != fp.piece() || !data.stuff.dst_candidates.contains(&pp.placement) {
                    continue;
                }
                let mut game = data.game.clone();
                game.state.falling_piece = Some(FallingPiece::new(fp.piece().default_spec(), pp.placement));
                game.lock()?;
                let mut pps = data.pps.clone();
                pps.remove(i);
                (game, pps)
            };
            tree::append_child(&node, NodeData::new(
                Some(Action::Move(MoveTransition::new(pp.placement.clone(), None))),
                game,
                pps,
            )?);
        }
    }

    let no_children = node.borrow().children.is_empty();
    if no_children {
        let can_hold = node.borrow().data.game.state.can_hold;
        if can_hold {
            let mut game = node.borrow().data.game.clone();
            game.hold()?;
            let pps = node.borrow().data.pps.clone();
            let child_data = NodeData::new(Some(Action::Hold), game, pps)?;
            tree::append_child(&node, child_data);
            return Ok(true);
        }
        return Ok(false);
    }

    Ok(true)
}

struct DecisionTree {
    root: Rc<RefCell<tree::Node<NodeData>>>,
}

impl DecisionTree {
    fn new(game: Game, pps: Vec<Rc<PiecePlacement>>) -> Result<Self, &'static str> {
        let root = tree::new(NodeData::new(None, game, pps)?);
        Ok(Self { root })
    }
}

// Vec<Vec<Action>>
// state managed with tree
// fn check(mut game: Game, pps: Vec<PiecePlacement>) -> Vec<Action> {
//     let mut r = Vec::new();
//     let mut remains = pps;
//     while !remains.is_empty() {
//         if game.state.falling_piece.is_none() {
//             if game.state.can_hold {
//                 r.push(Action::Hold);
//                 game.hold().unwrap();
//                 continue;
//             }
//             break;
//         }
//         let h = game.get_move_decision_helper().unwrap();
//         println!("=====================================");
//         println!("{}", game);
//         for pp in remains.iter() {
//             println!("{:?}", pp);
//         }
//         println!("---");
//         println!("{:?}", h.falling_piece.piece);
//         for dst in h.dst_candidates.iter() {
//             println!("{:?}", dst);
//         }
//         if let Some((i, p)) = remains.iter().enumerate()
//             .filter(|(_, pp)| pp.piece == h.falling_piece.piece && h.dst_candidates.contains(&pp.placement))
//             .map(|(i, pp)| (i, pp.placement.clone()))
//             .next() {
//             println!("---");
//             println!("=> {:?}", p);
//             r.push(Action::Move(MoveTransition::new(p.clone(), None)));
//             remains.remove(i);
//             game.state.falling_piece = Some(FallingPiece::new(h.falling_piece.piece.default_spec(), p));
//             game.lock().unwrap();
//         } else {
//             println!("---");
//             println!("=> not found");
//             if game.state.can_hold {
//                 r.push(Action::Hold);
//                 game.hold().unwrap();
//                 continue;
//             }
//             break;
//         }
//     }
//     if matches!(r.last(), Some(Action::Hold)) {
//         r.pop();
//     }
//     r
// }

macro_rules! pp {
    ($piece_name:ident, $orientation:literal, $x:literal, $y:literal) => {
        PiecePlacement::new(
            core::Piece::$piece_name,
            Placement::new(
                core::ORIENTATIONS[$orientation],
                grid::Vec2($x, $y),
            ),
        )
    }
}

#[derive(Copy, Clone, Debug)]
struct PiecePlacement {
    pub piece: Piece,
    pub placement: Placement,
}

impl PiecePlacement {
    fn new(piece: Piece, placement: Placement) -> Self {
        Self { piece, placement }
    }
}

fn main() {
    let tsd_opener_l_base = [
        pp!(I, 0, 2, -2),
        pp!(O, 0, 7, -1),
        pp!(L, 1, -1, 0),
    ];
    let tsd_opener_l_01 = tsd_opener_l_base.iter().copied().chain([
        pp!(S, 1, 5, 0),
        pp!(Z, 0, 3, 0),
        pp!(J, 2, 3, 2),
        pp!(T, 2, 1, 0),
    ].iter().copied()).collect::<Vec<_>>();

    let mut game: Game = Default::default();
    game.supply_next_pieces(&[
        Piece::I, Piece::S, Piece::Z, Piece::T, Piece::O, Piece::J, Piece::L,
        Piece::I, Piece::S, Piece::Z, Piece::T, Piece::O, Piece::J, Piece::L,
    ]);
    game.setup_falling_piece(None);
    let pps = tsd_opener_l_01.iter().copied().map(|pp| Rc::new(pp)).collect::<Vec<_>>();
    let mut t = DecisionTree::new(game, pps).unwrap();
    expand_node(&mut t.root).unwrap();

    tree::visit(&t.root, |node, state| {
        VisitPlan::Children
    });

    // let r = check(game, tsd_opener_l_01.to_vec());
    // println!("{:?}", r);
}
