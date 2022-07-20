use core::prelude::*;
use tree::arena::{NodeHandle, NodeArena, VecNodeArena};
use bot::Action;

struct NodeData {
    by_action: Option<Action>,
    game: Game<'static>,
    resource: MoveDecisionResource,
}

impl NodeData {
    pub fn new(by_action: Option<Action>, game: Game<'static>) -> Result<Self, &'static str> {
        let resource = MoveDecisionResource::with_game(&game)?;
        Ok(Self { by_action, game, resource })
    }
}

fn expand_node(arena: &mut VecNodeArena<NodeData>, node: NodeHandle) {
    if let Some(fp) = arena[node].data.game.state.falling_piece.clone() {
        // let pps_len = arena[node].data.pps.len();
        // for i in 0..pps_len {
        //     let pp = arena[node].data.pps.get(i).cloned().unwrap();
        //     let (game, pps) = {
        //         let data = &arena[node].data;
        //         if pp.piece != fp.piece() || !data.resource.dst_candidates.contains(&pp.placement) {
        //             continue;
        //         }
        //         let mut game = data.game.clone();
        //         game.state.falling_piece = Some(FallingPiece::new(fp.piece().default_spec(), pp.placement));
        //         game.lock().unwrap();
        //         let mut pps = data.pps.clone();
        //         pps.remove(i);
        //         (game, pps)
        //     };
        //     if game.state.falling_piece.is_some() {
        //         arena.append_child(node, NodeData::new(
        //             Some(Action::Move(MoveTransition::new(pp.placement.clone(), None))),
        //             game,
        //             pps,
        //         ).unwrap());
        //     }
        // }
    }
    if arena[node].data.game.state.can_hold {
        let mut game = arena[node].data.game.clone();
        game.hold().unwrap();
        if game.state.falling_piece.is_some() {
            // let pps = arena[node].data.pps.clone();
            // let child_data = NodeData::new(Some(Action::Hold), game, pps).unwrap();
            // arena.append_child(node, child_data);
        }
    }
}

fn main() {
    let mut initial_game: Game = Default::default();
    // TODO
    // initial_game.supply_next_pieces(args.pieces.as_slice());
    // {
    //     let mut n = args.pieces.len();
    //     if n < pps_len {
    //         let mut rpg = RandomPieceGenerator::new(StdRng::seed_from_u64(args.random_seed));
    //         while n < pps_len {
    //             let pieces = rpg.generate();
    //             initial_game.supply_next_pieces(&pieces);
    //             n += pieces.len();
    //         }
    //     }
    // }
    initial_game.setup_falling_piece(None).unwrap();
    println!("\n### Initial Game\n{}", initial_game);

    let mut arena: VecNodeArena<NodeData> = Default::default();
    let mut game = initial_game.clone();
    game.state.playfield.grid.disable_basic_grid();
    let root = arena.create(NodeData::new(None, game).unwrap());
}
