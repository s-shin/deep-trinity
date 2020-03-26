use core::{Game, Placement};
use super::Bot;

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot {}

impl Bot for SimpleBot {
    fn think(&mut self, game: &Game) -> Option<Placement> {
        let candidates = match game.get_move_candidates() {
            Ok(r) => r,
            _ => return None,
        };
        if candidates.is_empty() {
            return None;
        }
        let mut candidate = &candidates[0];
        for fp in &candidates {
            if fp.placement.pos.1 < candidate.placement.pos.1 {
                candidate = fp;
            }
        }
        Some(candidate.placement)
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleBot;
    use crate::test_bot;

    #[test]
    fn test_simple_bot() {
        let mut bot = SimpleBot::default();
        let seed = 0;
        // let seed = 409509985; // check circular problem
        let game = test_bot(&mut bot, seed, 100, false);
        assert!(game.stats.lock > 40);
    }
}
