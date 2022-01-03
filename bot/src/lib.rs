use std::error::Error;
use rand::SeedableRng;
use rand::rngs::StdRng;
use core::{MoveTransition, RandomPieceGenerator, MovePlayer, FallingPiece};

pub mod simple;
pub mod simple_tree;
pub mod mcts_puct;
pub mod tree;
pub mod template;

pub type Game = core::Game<'static>;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Move(MoveTransition),
    Hold,
}

pub trait Bot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>>;
}

pub fn test_bot<B: Bot>(bot: &mut B, random_seed: u64, max_iterations: usize, debug_print: bool) -> Result<Game, Box<dyn Error>> {
    let mut game: Game = Default::default();
    let mut pg = RandomPieceGenerator::new(StdRng::seed_from_u64(random_seed));
    game.supply_next_pieces(&pg.generate());
    game.setup_falling_piece(None).unwrap();

    for n in 0..max_iterations {
        if debug_print { println!("===== {} =====\n{}", n, game); }
        if game.should_supply_next_pieces() {
            game.supply_next_pieces(&pg.generate());
        }
        let action = bot.think(&game)?;
        if debug_print { println!("Action: {:?}", action); }
        match action {
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

//---

pub trait BotRunnerHook {
    fn on_start(&mut self, _game: &mut Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_iter(&mut self, _game: &mut Game) -> Result<bool, Box<dyn Error>> { Ok(true) }
    fn on_action(&mut self, _game: &Game, _action: &Action) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_action_step(&mut self, _game: &Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_end(&mut self, _game: &mut Game) -> Result<(), Box<dyn Error>> { Ok(()) }
}

pub struct BotRunner {
    max_iterations: usize,
    quick_action: bool,
    random_seed: Option<u64>,
    debug_print: bool,
}

impl BotRunner {
    pub fn new(max_iterations: usize, quick_action: bool, random_seed: Option<u64>, debug_print: bool) -> Self {
        Self { max_iterations, quick_action, random_seed, debug_print }
    }
    pub fn run(&self, bot: &mut impl Bot, mut hook: Option<&mut impl BotRunnerHook>) -> Result<Game, Box<dyn Error>> {
        let mut game: Game = Default::default();

        let mut rpg = self.random_seed.map(|seed| RandomPieceGenerator::new(StdRng::seed_from_u64(seed)));
        if let Some(rpg) = rpg.as_mut() {
            game.supply_next_pieces(&rpg.generate());
            game.setup_falling_piece(None).unwrap();
        }
        if let Some(hook) = hook.as_mut() {
            hook.on_start(&mut game)?;
        }

        for n in 0..self.max_iterations {
            if let Some(hook) = hook.as_mut() {
                if !hook.on_iter(&mut game)? {
                    break;
                }
            }
            if self.debug_print { println!("===== {} =====\n{}", n, game); }
            if let Some(rpg) = rpg.as_mut() {
                game.supply_next_pieces(&rpg.generate());
                game.setup_falling_piece(None).unwrap();
            }

            let action = bot.think(&game)?;
            if self.debug_print { println!("Action: {:?}", action); }
            if let Some(hook) = hook.as_mut() {
                hook.on_action(&game, &action)?;
            }

            match action {
                Action::Move(mt) => {
                    if self.quick_action {
                        let fp = FallingPiece::new_with_last_move_transition(
                            game.state.falling_piece.unwrap().piece_spec,
                            &mt,
                        );
                        game.state.falling_piece = Some(fp);
                        if self.debug_print { println!("{}", game); }
                        if let Some(hook) = hook.as_mut() {
                            hook.on_action_step(&game)?;
                        }
                    } else {
                        let path = game.get_almost_good_move_path(&mt)?;

                        let mut mp = MovePlayer::new(path);
                        while mp.step(&mut game)? {
                            if self.debug_print { println!("{}", game); }
                            if let Some(hook) = hook.as_mut() {
                                hook.on_action_step(&game)?;
                            }
                        }
                    }

                    game.lock().unwrap();
                    if game.state.is_game_over() {
                        break;
                    }
                }
                Action::Hold => {
                    game.hold()?;
                    if self.debug_print { println!("{}", game); }
                    if let Some(hook) = hook.as_mut() {
                        hook.on_action_step(&game)?;
                    }
                }
            }
        }

        if self.debug_print { println!("===== END =====\n{}", game); }
        if let Some(hook) = hook.as_mut() {
            hook.on_end(&mut game)?;
        }
        Ok(game)
    }
}
