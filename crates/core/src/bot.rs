use std::error::Error;
use rand::SeedableRng;
use rand::rngs::StdRng;
use crate::{Game, MoveTransition, RandomPieceGenerator, MovePlayer, FallingPiece};
use crate::helper::MoveDecisionResource;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    Move(MoveTransition),
    Hold,
}

pub trait Bot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>>;
}

//---

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let mdr = MoveDecisionResource::with_game(game)?;
        if mdr.dst_candidates.is_empty() {
            return Err("no movable placements".into());
        }
        let selected = mdr.dst_candidates.iter()
            .min_by(|pl1, pl2| pl1.pos.1.cmp(&pl2.pos.1))
            .unwrap();
        Ok(Action::Move(MoveTransition::new(selected.clone(), None)))
    }
}

//---

pub trait SimpleBotRunnerHooks {
    fn on_start(&mut self, _game: &Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_iter(&mut self, _game: &Game) -> Result<bool, Box<dyn Error>> { Ok(true) }
    fn on_action(&mut self, _game: &Game, _action: &Action) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_move_step(&mut self, _game: &Game) -> Result<(), Box<dyn Error>> { Ok(()) }
    fn on_end(&mut self, _game: &Game) -> Result<(), Box<dyn Error>> { Ok(()) }
}

pub struct DefaultSimpleBotRunnerHooks;

impl SimpleBotRunnerHooks for DefaultSimpleBotRunnerHooks {}

pub struct SimpleBotRunner {
    max_iterations: usize,
    quick_action: bool,
    random_seed: Option<u64>,
    debug_print: bool,
}

impl SimpleBotRunner {
    pub fn new(max_iterations: usize, quick_action: bool, random_seed: Option<u64>, debug_print: bool) -> Self {
        Self { max_iterations, quick_action, random_seed, debug_print }
    }
    pub fn run_with_no_hooks(&self, bot: &mut impl Bot) -> Result<Game, Box<dyn Error>> {
        let mut dummy = DefaultSimpleBotRunnerHooks;
        self.run(bot, &mut dummy)
    }
    pub fn run(&self, bot: &mut impl Bot, hook: &mut impl SimpleBotRunnerHooks) -> Result<Game, Box<dyn Error>> {
        let mut game: Game = Default::default();

        let mut rpg = self.random_seed.map(|seed| RandomPieceGenerator::new(StdRng::seed_from_u64(seed)));
        if let Some(rpg) = rpg.as_mut() {
            game.supply_next_pieces(&rpg.generate());
            game.setup_falling_piece(None).unwrap();
        }
        hook.on_start(&game)?;

        for n in 0..self.max_iterations {
            if !hook.on_iter(&game)? {
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
                        hook.on_move_step(&game)?;
                    } else {
                        let path = game.get_almost_good_move_path(&mt)?;

                        let mut mp = MovePlayer::new(path);
                        while mp.step(&mut game)? {
                            if self.debug_print { println!("{}", game); }
                            hook.on_move_step(&game)?;
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
                    hook.on_move_step(&game)?;
                }
            }
        }

        if self.debug_print { println!("===== END =====\n{}", game); }
        hook.on_end(&game)?;
        Ok(game)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_simple_bot_runner() {
        let mut runner = SimpleBotRunner::new(20, true, Some(0), false);
        let mut bot = SimpleBot::default();
        let game = runner.run_with_no_hooks(&mut bot).unwrap();
        // println!("{}", game);
        assert_eq!(20, game.stats.lock);
    }
}
