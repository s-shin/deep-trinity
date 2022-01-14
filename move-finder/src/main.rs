use std::rc::Rc;
use core::{Piece, Placement, MoveTransition, FallingPiece};
use core::helper::MoveDecisionStuff;
use bot::Action;
use tree::arena::{NodeArena, NodeHandle};

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

type VecNodeArena = tree::arena::VecNodeArena<NodeData>;

fn expand_node(arena: &mut VecNodeArena, node: NodeHandle) {
    if let Some(fp) = arena[node].data.game.state.falling_piece.clone() {
        let pps_len = arena[node].data.pps.len();
        for i in 0..pps_len {
            let pp = arena[node].data.pps.get(i).cloned().unwrap();
            let (game, pps) = {
                let data = &arena[node].data;
                if pp.piece != fp.piece() || !data.stuff.dst_candidates.contains(&pp.placement) {
                    continue;
                }
                let mut game = data.game.clone();
                game.state.falling_piece = Some(FallingPiece::new(fp.piece().default_spec(), pp.placement));
                game.lock().unwrap();
                let mut pps = data.pps.clone();
                pps.remove(i);
                (game, pps)
            };
            if game.state.falling_piece.is_some() {
                arena.append_child(node, NodeData::new(
                    Some(Action::Move(MoveTransition::new(pp.placement.clone(), None))),
                    game,
                    pps,
                ).unwrap());
            }
        }
    }
    if arena[node].data.game.state.can_hold {
        let mut game = arena[node].data.game.clone();
        game.hold().unwrap();
        if game.state.falling_piece.is_some() {
            let pps = arena[node].data.pps.clone();
            let child_data = NodeData::new(Some(Action::Hold), game, pps).unwrap();
            arena.append_child(node, child_data);
        }
    }
}

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
    let debug_trace = false;

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

    let pps = tsd_opener_l_01.iter().copied().map(|pp| Rc::new(pp)).collect::<Vec<_>>();
    let pps_len = pps.len();
    println!("## Positions");
    for ps in pps.iter() {
        println!("{} {} {}", ps.piece.char(), ps.placement.orientation.id(), ps.placement.pos);
    }

    let mut game: Game = Default::default();
    game.supply_next_pieces(&[
        Piece::I, Piece::S, Piece::Z, Piece::T, Piece::O, Piece::J, Piece::L,
        Piece::I, Piece::S, Piece::Z, Piece::T, Piece::O, Piece::J, Piece::L,
    ]);
    game.setup_falling_piece(None);
    println!("## Game\n{}", game);

    let mut arena = VecNodeArena::default();
    let root = arena.create(NodeData::new(None, game, pps).unwrap());

    let mut open = vec![root];
    while !open.is_empty() {
        let target = open.pop().unwrap();
        expand_node(&mut arena, target);
        open.extend(arena[target].children());
    }
    if debug_trace {
        arena.visit_depth_first(root, |arena, node, ctx| {
            let indent = "  ".repeat(ctx.depth);
            let n = &arena[node];
            println!("{}- by_action: {:?}", indent, n.data.by_action);
            println!("{}  game: |-\n{}", indent, n.data.game.to_string().split("\n")
                .map(|line| format!("{}    {}", indent, line)).collect::<Vec<_>>().join("\n"));
            println!("{}  children: {}", indent, if n.is_leaf() { "[]" } else { "" });
        });
    }

    println!("## Result");

    let mut found = Vec::new();
    arena.visit_depth_first(root, |arena, node, ctx| {
        if arena[node].data.game.stats.lock == pps_len as u32 {
            found.push(arena.route(node));
            ctx.skip();
        }
    });

    for (i, route) in found.iter().enumerate() {
        println!("--- {} ---", i);
        for n in route.iter() {
            if let Some(action) = arena[*n].data.by_action {
                println!("{:?}", action);
            }
        }
    }
}
