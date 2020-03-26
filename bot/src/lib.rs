use core::{Game, Placement};
use std::error::Error;

pub mod simple;
pub mod simple2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    MoveTo(Placement),
    Hold,
}

pub trait Bot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>>;
}

pub fn test_bot<B: Bot>(bot: &mut B, random_seed: u64, max_iterations: usize, debug_print: bool) -> Result<Game, Box<dyn Error>> {
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
        match bot.think(&game)? {
            Action::MoveTo(dst) => {
                let fp = game.state.falling_piece.as_ref().unwrap();
                let dst2 = core::get_nearest_placement_alias(fp.piece, &dst, &fp.placement, None);
                if debug_print { println!("{:?}: {:?} or {:?}", fp.piece, dst, dst2); }

                let mut rec = None;
                for i in 0..=2 {
                    // For special rotations, we should also check original destination.
                    let dst = match i {
                        0 => &dst2,
                        1 => &dst2,
                        2 => &dst,
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
                while mp.step(&mut game)? {
                    if debug_print { println!("{}", game); }
                }

                game.lock().unwrap();
                if game.state.is_game_over() {
                    break;
                }
            }
            Action::Hold => {
                game.hold()?;
                if debug_print { println!("{}", game); }
            }
        }
    }
    if debug_print {
        println!("===== END =====");
        println!("{}", game);
    }
    Ok(game)
}
