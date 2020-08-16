use core::{Game, Placement, Piece, Grid};
use crate::{Action, Bot};
use std::error::Error;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};

fn verify_c4w_x(pf_width: i8, piece: Piece, placement: Placement) -> bool {
    let x = placement.pos.0;
    match piece {
        Piece::I => {
            match placement.orientation {
                core::ORIENTATION_1 | core::ORIENTATION_3 => (-2 <= x && x <= 0) || (pf_width - 4 <= x && x <= pf_width - 2),
                _ => false,
            }
        }
        Piece::O => {
            if x == 0 || x == pf_width - 3 {
                true
            } else {
                match placement.orientation {
                    core::ORIENTATION_0 | core::ORIENTATION_1 => x == -1 || x == pf_width - 4,
                    core::ORIENTATION_2 | core::ORIENTATION_3 => x == 1 || x == pf_width - 2,
                    _ => false,
                }
            }
        }
        _ => {
            if x == 0 || x == pf_width - 3 {
                true
            } else {
                match placement.orientation {
                    core::ORIENTATION_1 => x == -1 || x == pf_width - 4,
                    core::ORIENTATION_3 => x == 1 || x == pf_width - 2,
                    _ => false,
                }
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
struct Advanced;

impl Bot for Advanced {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let candidates = game.get_move_candidates()?;
        assert!(!candidates.is_empty());

        let candidates = candidates.iter()
            .filter(|mt| verify_c4w_x(game.state.playfield.grid.width() as i8, game.state.falling_piece.as_ref().unwrap().piece, mt.placement))
            .collect::<Vec<_>>();

        let n = rng.gen_range(0, candidates.len());
        let selected = candidates.iter().nth(n).unwrap();
        Ok(Action::Move(**selected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bot;

    #[test]
    fn test_advanced_bot() {
        let spec = core::PieceSpec::of(core::Piece::J);
        println!("{}", spec.grids[0].basic_grid);
        println!("{}", spec.grids[1].basic_grid);
        println!("{}", spec.grids[2].basic_grid);
        println!("{}", spec.grids[3].basic_grid);

        let mut bot = Advanced::default();
        let seed = 0;
        let game = test_bot(&mut bot, seed, 10, true).unwrap();
        assert!(game.stats.lock > 40);
    }
}
