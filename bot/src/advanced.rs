use core::{Game, Placement, Piece, Grid, FallingPiece, MoveTransition, LineClear, TSpinJudgementMode};
use crate::{Action, Bot};
use std::error::Error;
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::{HashMap, HashSet};
use rand::seq::{SliceRandom, IteratorRandom};

fn check_opener(game: &Game) -> bool {
    let count = game.stats.lock + game.state.hold_piece.map_or(0, |_| 1);
    count & 7 == 0 && game.state.playfield.is_empty()
}

fn check_piece_order(game: &Game, order: &[Piece]) -> bool {
    debug_assert!(game.state.falling_piece.is_some());
    debug_assert!(!order.is_empty());
    let mut i = if game.state.falling_piece.as_ref().unwrap().piece == *order.get(0).unwrap() { 1 } else { 0 };
    for p in game.state.next_pieces.iter().take(game.state.next_pieces.visible_num) {
        if let Some(idx) = order.iter().position(|pp| *pp == *p) {
            if i < idx {
                return false;
            }
            i += 1;
            if i == order.len() {
                break;
            }
        }
    }
    true
}

#[derive(Debug, Clone)]
struct PlacementCondition {}

#[derive(Debug, Clone)]
enum Move {
    Hold,
    To(Placement),
    ToAny(PlacementCondition),
}

#[derive(Debug, Clone)]
struct MoveDirection([Vec<Move>; 7]);

impl MoveDirection {
    fn get(&self, p: Piece) -> Option<&Vec<Move>> {
        self.0.iter().nth(p.to_usize())
    }
}

#[derive(Debug, Clone, Default)]
struct MoveDirector {
    cursor: [usize; 7],
}

impl MoveDirector {
    pub fn next(&mut self, direction: &MoveDirection, piece: Piece) -> Option<Move> {
        let idx: usize = self.cursor[piece.to_usize()];
        if let Some(mvs) = direction.get(piece.into()) {
            if let Some(mv) = mvs.get(idx) {
                self.cursor[piece.to_usize()] = idx + 1;
                Some(mv.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
struct EvaluateResult {
    // expected_line_clear: Option<LineClear>,
    // expected_num_blocks: u8,
    move_direction: MoveDirection,
}

impl EvaluateResult {
    fn new(move_direction: MoveDirection) -> Self {
        Self { move_direction }
    }
}

fn evaluate_tsd_opener_r(game: &Game) -> Option<EvaluateResult> {
    if !check_opener(game) {
        return None;
    }
    if !(check_piece_order(game, &[Piece::O, Piece::J]) && check_piece_order(game, &[Piece::I, Piece::S])) {
        return None;
    }
    let direction = MoveDirection([
        // S
        vec![Move::To(Placement::new(core::ORIENTATION_0, (4, 0).into()))],
        // Z
        vec![Move::To(Placement::new(core::ORIENTATION_1, (1, 0).into()))],
        // L
        vec![Move::To(Placement::new(core::ORIENTATION_1, (-1, 2).into()))],
        // J
        vec![Move::To(Placement::new(core::ORIENTATION_3, (8, 0).into()))],
        // I
        vec![Move::To(Placement::new(core::ORIENTATION_0, (2, -2).into()))],
        // T
        vec![Move::Hold],
        // O
        vec![Move::To(Placement::new(core::ORIENTATION_0, (-1, -1).into()))],
    ]);
    Some(EvaluateResult::new(direction))
}

fn check_c4w_x(pf_width: i8, piece: Piece, placement: Placement) -> bool {
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

struct GameData<'a> {
    game: &'a Game,
    current_piece: Piece,
    move_candidates: HashSet<MoveTransition>,
}

impl<'a> GameData<'a> {
    fn new(game: &'a Game) -> Result<Self, Box<dyn Error>> {
        debug_assert!(game.state.falling_piece.is_some());
        let current_piece = game.state.falling_piece.as_ref().unwrap().piece;
        let move_candidates = game.get_move_candidates()?;
        debug_assert!(!move_candidates.is_empty());
        Ok(Self { game, current_piece, move_candidates })
    }
    fn search_tspin(&self) -> Vec<(&MoveTransition, LineClear)> {
        if self.current_piece != Piece::T {
            return vec![];
        }
        let mut candidates = vec![];
        for mt in self.move_candidates.iter() {
            if mt.hint.is_none() {
                continue;
            }
            let line_clear = self.game.state.playfield.check_line_clear(
                &FallingPiece::new_with_last_move_transition(Piece::T, mt),
                TSpinJudgementMode::PuyoPuyoTetris);
            if line_clear.tspin.is_none() {
                continue;
            }
            candidates.push((mt, line_clear));
        }
        candidates
    }
    fn search_tetris(&self) -> Option<&MoveTransition> {
        if self.current_piece != Piece::I {
            return None;
        }
        for mt in self.move_candidates.iter() {
            if mt.placement.orientation.is_even() {
                continue;
            }
            let line_clear = self.game.state.playfield.check_line_clear(
                &FallingPiece::new(self.current_piece, mt.placement),
                TSpinJudgementMode::PuyoPuyoTetris);
            if line_clear.is_tetris() {
                return Some(mt);
            }
        }
        None
    }
}

#[derive(Clone, Debug)]
struct Advanced {
    direction: Option<MoveDirection>,
    director: Option<MoveDirector>,
    rng: StdRng,
}

impl Advanced {
    fn new() -> Self {
        Self {
            direction: None,
            director: None,
            rng: StdRng::seed_from_u64(0),
        }
    }
}

impl Bot for Advanced {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let data = GameData::new(game)?;
        let game_after_hold = if game.state.can_hold && game.state.hold_piece.is_some() {
            let mut game = game.clone();
            game.hold()?;
            Some(game)
        } else {
            None
        };
        let data_after_hold = if let Some(game) = &game_after_hold {
            Some(GameData::new(game)?)
        } else {
            None
        };

        let current_piece = game.state.falling_piece.as_ref().unwrap().piece;
        let tspin_candidates = data.search_tspin();
        let tspin_candidates_after_hold = data_after_hold.as_ref().map(|data| data.search_tspin());
        let tetris_candidate  = data.search_tetris();
        let tetris_candidate_after_hold = data_after_hold.as_ref().map(|data| data.search_tetris()).flatten();
        println!("!!! tspin_candidates: {:?}", tspin_candidates);
        println!("!!! tetris_candidate: {:?}", tetris_candidate);
        println!("!!! tspin_candidates_after_hold: {:?}", tspin_candidates_after_hold);
        println!("!!! tetris_candidate_after_hold: {:?}", tetris_candidate_after_hold);

        let mv = if let Some(director) = self.director.as_mut() {
            director.next(self.direction.as_ref().unwrap(), current_piece)
        } else {
            self.direction = None;
            self.director = None;
            println!("!!! evaluate_tsd_opener_r");
            if let Some(r) = evaluate_tsd_opener_r(game) {
                println!("!!! tsd_opener_r found: {:?}", r);
                self.direction = Some(r.move_direction);
                self.director = Some(Default::default());
            }
            if let Some(director) = self.director.as_mut() {
                director.next(self.direction.as_ref().unwrap(), current_piece)
            } else {
                None
            }
        };
        println!("!!! mv: {:?}", mv);
        if let Some(mv) = mv {
            match mv {
                Move::Hold => {
                    return Ok(Action::Hold);
                }
                Move::To(placement) => {
                    let mt = data.move_candidates.iter()
                        .filter(|mt| mt.placement == placement || core::get_placement_aliases(current_piece, &mt.placement).iter().find(|p| **p == placement).is_some())
                        .take(1)
                        .next()
                        .unwrap();
                    return Ok(Action::Move(*mt));
                }
                Move::ToAny(cond) => {
                    panic!("todo");
                }
            }
        }

        if let Some(candidates) = tspin_candidates_after_hold.as_ref() {
            if !candidates.is_empty() {
                return Ok(Action::Hold);
            }
        }
        if !tspin_candidates.is_empty() {
            let (mt, _) = tspin_candidates.choose(&mut self.rng).unwrap();
            return Ok(Action::Move(**mt));
        }
        if !tetris_candidate_after_hold.is_some() {
            return Ok(Action::Hold);
        }
        if let Some(mt) = tetris_candidate {
            return Ok(Action::Move(*mt));
        }

        let mt = data.move_candidates.iter().choose(&mut self.rng).unwrap();
        Ok(Action::Move(*mt))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bot;

    #[test]
    fn test_advanced_bot() {
        let mut bot = Advanced::new();
        let seed = 6;
        let _game = test_bot(&mut bot, seed, 10, true).unwrap();
        // assert!(game.stats.lock > 40);
    }
}
