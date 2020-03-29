use core::{Game, Placement, MoveTransition};
use std::error::Error;

pub mod simple;
pub mod simple_tree;
pub mod puct;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Move(MoveTransition),
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
            Action::Move(mt) => {
                let path = game.get_almost_good_move_path(&mt)?;

                let mut mp = MovePlayer::new(path);
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
