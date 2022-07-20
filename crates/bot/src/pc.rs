use crate::{Bot, Action};
use core::Game;
use core::helper::MoveDecisionResource;
use std::error::Error;

#[derive(Copy, Clone, Debug, Default)]
pub struct PcBot {}

impl Bot for PcBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let mdm = MoveDecisionResource::with_game(game)?;
        if mdm.dst_candidates.is_empty() {
            return Err("no movable placements".into());
        }

        return Err("todo".into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BotRunner;

    #[test]
    fn test_simple_bot() {
        let seed = 0;
        let runner = BotRunner::new(100, true, Some(seed), false);
        let mut bot = PcBot::default();
        let game = runner.run_with_no_hooks(&mut bot).unwrap();
        // let game = test_bot(&mut bot, seed, 100, false).unwrap();
        assert!(game.stats.lock > 40);
    }
}
