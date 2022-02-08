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

//---

pub trait BotRunnerHooks {
    fn on_start(&mut self, _game: &mut Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_iter(&mut self, _game: &mut Game) -> Result<bool, Box<dyn Error>> { Ok(true) }
    fn on_action(&mut self, _game: &Game, _action: &Action) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_action_step(&mut self, _game: &Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_end(&mut self, _game: &mut Game) -> Result<(), Box<dyn Error>> { Ok(()) }
}

pub struct DummyBotRunnerHooks;

impl BotRunnerHooks for DummyBotRunnerHooks {}

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
    pub fn run_with_no_hooks(&self, bot: &mut impl Bot) -> Result<Game, Box<dyn Error>> {
        let mut dummy = DummyBotRunnerHooks;
        self.run(bot, &mut dummy)
    }
    pub fn run(&self, bot: &mut impl Bot, hook: &mut impl BotRunnerHooks) -> Result<Game, Box<dyn Error>> {
        let mut game: Game = Default::default();

        let mut rpg = self.random_seed.map(|seed| RandomPieceGenerator::new(StdRng::seed_from_u64(seed)));
        if let Some(rpg) = rpg.as_mut() {
            game.supply_next_pieces(&rpg.generate());
            game.setup_falling_piece(None).unwrap();
        }
        hook.on_start(&mut game)?;

        for n in 0..self.max_iterations {
            if !hook.on_iter(&mut game)? {
                break;
            }
            if self.debug_print { println!("===== {} =====\n{}", n, game); }
            if game.should_supply_next_pieces() {
                if let Some(rpg) = rpg.as_mut() {
                    game.supply_next_pieces(&rpg.generate());
                }
            }

            let action = bot.think(&game)?;
            if self.debug_print { println!("Action: {:?}", action); }
            hook.on_action(&game, &action)?;

            match action {
                Action::Move(mt) => {
                    if self.quick_action {
                        let fp = FallingPiece::new_with_last_move_transition(
                            game.state.falling_piece.unwrap().piece_spec,
                            &mt,
                        );
                        game.state.falling_piece = Some(fp);
                        if self.debug_print { println!("{}", game); }
                        hook.on_action_step(&game)?;
                    } else {
                        let path = game.get_almost_good_move_path(&mt)?;

                        let mut mp = MovePlayer::new(path);
                        while mp.step(&mut game)? {
                            if self.debug_print { println!("{}", game); }
                            hook.on_action_step(&game)?;
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
                    hook.on_action_step(&game)?;
                }
            }
        }

        if self.debug_print { println!("===== END =====\n{}", game); }
        hook.on_end(&mut game)?;
        Ok(game)
    }
}
