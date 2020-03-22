use core::*;

pub trait Bot {
    fn think(&mut self, game: &Game) -> Option<Placement>;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Option<Placement> {
        if game.state.falling_piece.is_none() {
            return None;
        }
        let pf = &game.state.playfield;
        let fp = game.state.falling_piece.as_ref().unwrap();
        let lockable = pf.search_lockable_placements(fp.piece);
        let search_result = game.search_moves(
            &mut move_search::bruteforce::BruteForceMoveSearcher::default());
        debug_assert!(search_result.is_ok());
        let search_result = search_result.unwrap();
        let candidates = lockable.iter().filter(|p| { search_result.contains(p) }).collect::<Vec<&Placement>>();
        if candidates.is_empty() {
            return None;
        }
        let mut r = *candidates[0];
        for p in candidates {
            if r.pos.1 > p.pos.1 {
                r = *p;
            }
        }
        Some(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_simple_bot() {
        let debug = false;
        let mut game = Game::new(Default::default());
        let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(0));
        game.supply_next_pieces(&pg.generate());
        game.setup_falling_piece(None).unwrap();

        let mut bot = SimpleBot::default();
        for _ in 0..50 {
            if debug { println!("{}", game); }
            if game.should_supply_next_pieces() {
                game.supply_next_pieces(&pg.generate());
            }
            let dst = bot.think(&game);
            if dst.is_none() {
                break;
            }
            let dst = dst.unwrap();

            let fp = game.state.falling_piece.as_ref().unwrap();
            let dst2 = core::get_nearest_placement_alias(fp.piece, &dst, &fp.placement, None);
            if debug { println!("{:?}: {:?} => {:?}", fp.piece, dst, dst2); }

            // let mut searcher = move_search::bruteforce::BruteForceMoveSearcher::default();
            let mut searcher = move_search::humanly_optimized::HumanlyOptimizedMoveSearcher::new(dst, true);
            let ret = game.search_moves(&mut searcher).unwrap();
            let rec = ret.get(&dst2).unwrap_or_else(|| {
                if debug { println!("fallback"); }
                let mut searcher = move_search::astar::AStarMoveSearcher::new(dst2, false);
                let ret = game.search_moves(&mut searcher).unwrap();
                ret.get(&dst2).unwrap()
            });
            let mut mp = MovePlayer::new(rec);
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
