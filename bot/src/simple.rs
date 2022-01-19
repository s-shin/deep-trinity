use crate::{Bot, Action};
use core::Game;
use std::error::Error;

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let candidates = game.get_move_candidates()?;
        if candidates.is_empty() {
            return Err("no movable placements".into());
        }
        let selected = candidates.iter()
            .min_by(|mt1, mt2| mt1.placement.pos.1.cmp(&mt2.placement.pos.1))
            .unwrap();
        Ok(Action::Move(selected.clone()))
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
        let mut bot = SimpleBot::default();
        let game = runner.run_with_no_hook(&mut bot).unwrap();
        // let game = test_bot(&mut bot, seed, 100, false).unwrap();
        assert!(game.stats.lock > 40);
    }
}
