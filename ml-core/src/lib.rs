use std::collections::HashMap;
use rand::prelude::StdRng;
use rand::SeedableRng;
use core::Grid;

#[cfg(feature = "async_session")]
pub mod async_session;

pub const HOLD_ACTION_ID: u32 = 0;
pub const NUM_ACTIONS: u32 = 1 + 10 * 30 * 4 * 2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Action(pub u32);

impl Action {
    // hold, (x, y, orientation, is_rotated) * 10 * 30 * 4 * 2
    pub fn from_move_transition(mt: &core::MoveTransition, piece: core::Piece) -> Self {
        let offset = if piece == core::Piece::I { 2 } else { 1 } as i32;
        let x = mt.placement.pos.0 as i32 + offset;
        assert_debug!(0 <= x);
        assert_debug!(x < 10);
        let x = (x * 30 * 4 * 2) as u32;
        let y = mt.placement.pos.1 as i32 + offset;
        assert_debug!(0 <= y);
        assert_debug!(y < 30);
        let y = (y * 4 * 2) as u32;
        let o = mt.placement.orientation.id() as u32 * 2;
        let r = if let Some(hint) = mt.hint {
            if let core::Move::Rotate(_) = hint.by { 1 } else { 0 }
        } else { 0 } as u32;
        let id = 1 + x + y + o + r;
        Self(id)
    }
    pub fn is_hold(&self) -> bool { self.0 == HOLD_ACTION_ID }
}

pub fn calc_reward(stats: &core::Statistics) -> f32 {
    use core::{StatisticsEntryType, LineClear, TSpin};
    let mut reward = 0.0;
    for (ent_type, val) in &[
        (StatisticsEntryType::LineClear(LineClear::new(1, None)), 0.1),
        (StatisticsEntryType::LineClear(LineClear::new(2, None)), 1.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, None)), 2.0),
        (StatisticsEntryType::LineClear(LineClear::new(4, None)), 4.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Standard))), 2.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Standard))), 4.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, Some(TSpin::Standard))), 6.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Mini))), 0.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Mini))), 1.0),
        (StatisticsEntryType::PerfectClear, 10.0),
    ] {
        reward += stats.get(*ent_type) as f32 * val;
    }
    if stats.btb.max() > 0 {
        reward += 1.0;
    }
    reward += match stats.combo.max() {
        0 | 1 => 0.0,
        2 | 3 => 1.0,
        4 | 5 => 2.0,
        6 | 7 => 3.0,
        8 | 9 | 10 => 4.0,
        _ => 5.0,
    };
    const MAX: f32 = 22.0;
    if reward > MAX { 1.0 } else { reward / MAX }
}

#[derive(Clone, Debug)]
pub struct GameSession {
    piece_gen: core::RandomPieceGenerator<StdRng>,
    game: core::Game,
    legal_actions: HashMap<Action, core::MoveTransition>,
    last_reward: f32,
}

impl GameSession {
    pub fn new(rand_seed: Option<u64>) -> Result<Self, &'static str> {
        let rng = if let Some(seed) = rand_seed { StdRng::seed_from_u64(seed) } else { StdRng::from_entropy() };
        let mut pg = core::RandomPieceGenerator::new(rng);
        let mut game: core::Game = Default::default();
        game.supply_next_pieces(&pg.generate());
        game.setup_falling_piece(None).unwrap();
        let mut r = Self {
            piece_gen: pg,
            game,
            legal_actions: HashMap::new(),
            last_reward: 0.0,
        };
        r.sync()?;
        Ok(r)
    }
    pub fn reset(&mut self, rand_seed: Option<u64>) -> Result<(), &'static str> {
        if let Some(seed) = rand_seed {
            self.piece_gen = core::RandomPieceGenerator::new(StdRng::seed_from_u64(seed));
        }
        self.game = Default::default();
        self.game.supply_next_pieces(&self.piece_gen.generate());
        self.game.setup_falling_piece(None).unwrap();
        self.last_reward = 0.0;
        self.sync()?;
        Ok(())
    }
    fn sync(&mut self) -> Result<(), &'static str> {
        let piece = self.game.state.falling_piece.as_ref().unwrap().piece;
        let mut legal_actions = HashMap::new();
        let candidates = self.game.get_move_candidates()?;
        for mt in candidates.iter() {
            legal_actions.insert(Action::from_move_transition(mt, piece), *mt);
        }
        self.legal_actions = legal_actions;
        Ok(())
    }
    pub fn step(&mut self, action: Action) -> Result<(), &'static str> {
        if action.is_hold() {
            self.game.hold()?;
            self.last_reward = 0.0;
        } else {
            let mt = self.legal_actions.get(&action).unwrap();
            let piece = self.game.state.falling_piece.as_ref().unwrap().piece;
            let fp = core::FallingPiece::new_with_last_move_transition(piece, &mt);
            self.game.state.falling_piece = Some(fp);
            let stats = self.game.stats.clone();
            self.game.lock()?;
            let stats = self.game.stats.clone() - stats;
            self.last_reward = calc_reward(&stats);
        }
        if self.game.should_supply_next_pieces() {
            self.game.supply_next_pieces(&self.piece_gen.generate());
        }
        self.sync()?;
        Ok(())
    }
    pub fn game_str(&self) -> String { format!("{}", self.game) }
    pub fn legal_actions(&self) -> Vec<u32> {
        let mut r = self.legal_actions.keys().map(|a| a.0).collect::<Vec<_>>();
        if self.game.state.can_hold {
            r.push(HOLD_ACTION_ID);
        }
        r
    }
    pub fn observation(&self) -> Vec<u32> {
        let state = &self.game.state;
        let fp = state.falling_piece.as_ref().unwrap();
        let mut r = Vec::with_capacity(state.playfield.grid.height() as usize + 2);
        // [rows[n], rows[n+1]] * 20
        r.resize(state.playfield.grid.height() as usize / 2, 0 as u32);
        for (i, row) in state.playfield.grid.bit_grid.rows.iter().enumerate() {
            r[i / 2] += (*row as u32) << (16 * (i % 2));
        }
        // can_hold(2), hold_piece(8), falling_piece(7)
        r.push(
            if state.can_hold { 1 } else { 0 }
                + if let Some(p) = state.hold_piece { p as u32 + 1 } else { 0 } * 2
                + fp.piece as u32 * 2 * 8
        );
        // next * N
        r.push(
            state.next_pieces.pieces.iter()
                .take(state.next_pieces.visible_num)
                .enumerate()
                .fold(0 as u32, |acc, (i, p)| {
                    acc + (*p as u32) * (7 * i as u32)
                })
        );
        r
    }
    pub fn last_reward(&self) -> f32 { self.last_reward }
    pub fn is_done(&self) -> bool { self.game.state.is_game_over() || self.legal_actions.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_reward() {
        let stats: core::Statistics = Default::default();
        let mut stats2 = stats.clone();
        stats2.line_clear.add(&core::LineClear::new(1, None), 1);
        let diff = stats2 - stats;
        assert!(calc_reward(&diff) > 0.0);
    }
}
