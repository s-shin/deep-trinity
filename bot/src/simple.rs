use core::{Game, Placement, move_search};
use super::Bot;

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Option<Placement> {
        let candidates = match game.get_move_candidates() {
            Ok(r) => r,
            _ => return None,
        };
        if candidates.is_empty() {
            return None;
        }
        let mut candidate = &candidates[0];
        for fp in &candidates {
            if fp.placement.pos.1 < candidate.placement.pos.1 {
                candidate = fp;
            }
        }
        Some(candidate.placement)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use core::{RandomPieceGenerator, MovePlayer};

    #[test]
    fn test_simple_bot() {
        let debug = false;
        // let seed = 0;
        let seed = 409509985; // check circular problem
        let mut game = Game::new(Default::default());
        let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(seed));
        game.supply_next_pieces(&pg.generate());
        game.setup_falling_piece(None).unwrap();

        let mut bot = SimpleBot::default();
        for n in 0..100 {
            if debug { println!("===== {} =====\n{}", n, game); }
            if game.should_supply_next_pieces() {
                game.supply_next_pieces(&pg.generate());
            }
            let dst = bot.think(&game);
            if dst.is_none() {
                break;
            }
            let dst1 = dst.unwrap();
            let fp = game.state.falling_piece.as_ref().unwrap();
            let dst2 = core::get_nearest_placement_alias(fp.piece, &dst1, &fp.placement, None);
            if debug { println!("{:?}: {:?} or {:?}", fp.piece, dst1, dst2); }

            let mut rec = None;
            for i in 0..=2 {
                // For special rotations, we should also check original destination.
                let dst = match i {
                    0 => &dst2,
                    1 => &dst2,
                    2 => &dst1,
                    _ => panic!(),
                };
                let ret = match i {
                    0 => game.search_moves(&mut move_search::humanly_optimized::HumanlyOptimizedMoveSearcher::new(*dst, true)),
                    1 => game.search_moves(&mut move_search::astar::AStarMoveSearcher::new(*dst, false)),
                    2 => game.search_moves(&mut move_search::astar::AStarMoveSearcher::new(*dst, false)),
                    _ => panic!(),
                };
                if let Some(r) = ret.unwrap().get(dst) {
                    rec = Some(r);
                    break;
                }
            }

            let mut mp = MovePlayer::new(rec.unwrap());
            while mp.step(&mut game).unwrap() {
                if debug { println!("{}", game); }
            }

            game.lock().unwrap();
            if game.state.is_game_over() {
                break;
            }
        }
        if debug { println!("{}", game); }
        assert!(game.stats.lock > 40);
    }
}
