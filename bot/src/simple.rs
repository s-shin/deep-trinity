use core::Game;
use crate::{Bot, Action};
use std::error::Error;

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let candidates = game.get_move_candidates()?;
        if candidates.is_empty() {
            return Err("no movable placements".into());
        }
        let mut candidate = &candidates[0];
        for fp in &candidates {
            if fp.placement.pos.1 < candidate.placement.pos.1 {
                candidate = fp;
            }
        }
        Ok(Action::Move(candidate.placement))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bot;

    #[test]
    fn test_simple_bot() {
        let mut bot = SimpleBot::default();
        let seed = 0;
        // let seed = 409509985; // check circular problem
        let game = test_bot(&mut bot, seed, 100, false).unwrap();
        assert!(game.stats.lock > 40);
    }
}
