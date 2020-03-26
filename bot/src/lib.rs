use core::{Game, Placement};

pub mod simple;
pub mod simple2;

pub trait Bot {
    fn think(&mut self, game: &Game) -> Option<Placement>;
}

pub fn test_bot<B: Bot>(bot: &mut B, random_seed: u64, max_iterations: usize, debug_print: bool) -> Game {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use core::{RandomPieceGenerator, MovePlayer, move_search};

    let mut game = Game::new(Default::default());
    let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(random_seed));
    game.supply_next_pieces(&pg.generate());
    game.setup_falling_piece(None).unwrap();

    for n in 0..max_iterations {
        if debug_print { println!("===== {} =====\n{}", n, game); }
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
        if debug_print { println!("{:?}: {:?} or {:?}", fp.piece, dst1, dst2); }

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
            if debug_print { println!("{}", game); }
        }

        game.lock().unwrap();
        if game.state.is_game_over() {
            break;
        }
    }
    if debug_print {
        println!("===== END =====");
        println!("{}", game);
    }
    game
}
