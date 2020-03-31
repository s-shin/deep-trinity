extern crate pyo3;
extern crate core;

use std::collections::HashMap;
use pyo3::prelude::*;
use rand::prelude::StdRng;
use rand::SeedableRng;
use core::Placement;

fn to_py_err(e: &'static str) -> PyErr {
    pyo3::exceptions::RuntimeError::py_err(e)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
struct Action(u32);

impl Action {
    fn from_move_transition(mt: &core::MoveTransition) -> Self {
        // TODO
        Self(0)
    }
    fn is_hold(&self) -> bool {
        // TODO
        false
    }
    fn as_move_transaction(&self) -> core::MoveTransition {
        // TODO
        core::MoveTransition::new(Placement::default(), None)
    }
}

#[pyclass]
struct Environment {
    game: Option<core::Game>,
    piece_gen: core::RandomPieceGenerator<StdRng>,
    legal_actions: Option<HashMap<Action, core::MoveTransition>>,
}

#[pymethods]
impl Environment {
    #[new]
    fn new() -> Self {
        Self {
            game: None,
            piece_gen: core::RandomPieceGenerator::new(StdRng::seed_from_u64(0)),
            legal_actions: None,
        }
    }
    fn update_caches(&mut self) -> PyResult<()> {
        if self.game.is_none() {
            self.legal_actions = None;
            return Ok(());
        }
        let mut legal_actions = HashMap::new();
        let candidates = self.game.as_ref().unwrap().get_move_candidates().map_err(to_py_err)?;
        for mt in candidates.iter() {
            legal_actions.insert(Action::from_move_transition(mt), *mt);
        }
        self.legal_actions = Some(legal_actions);
        Ok(())
    }
    pub fn to_string(&self) -> String {
        if let Some(game) = self.game.as_ref() {
            format!("{}", game)
        } else {
            "".into()
        }
    }
    pub fn legal_actions(&self) -> PyResult<Vec<u32>> {
        if self.legal_actions.is_none() {
            return Err(to_py_err("no legal actions"));
        }
        Ok(self.legal_actions.as_ref().unwrap().keys().map(|a| a.0).collect::<Vec<_>>())
    }
    pub fn reset(&mut self, rand_seed: Option<u64>) -> PyResult<()> {
        self.piece_gen = core::RandomPieceGenerator::new(if let Some(seed) = rand_seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_entropy()
        });
        let mut game: core::Game = Default::default();
        game.supply_next_pieces(&self.piece_gen.generate());
        game.setup_falling_piece(None).unwrap();
        self.game = Some(game);
        self.update_caches()?;
        Ok(())
    }
    pub fn step(&mut self, action_id: u32) -> PyResult<bool> {
        if self.game.is_none() {
            return Err(to_py_err("game not initialized"));
        }
        let game = self.game.as_mut().unwrap();
        let action = Action(action_id);
        if action.is_hold() {
            game.hold().map_err(to_py_err)?;
        } else {
            let mt = Action(action_id).as_move_transaction();
            let piece = game.state.falling_piece.as_ref().unwrap().piece;
            let fp = core::FallingPiece::new_with_last_move_transition(piece, &mt);
            game.state.falling_piece = Some(fp);
            game.lock().map_err(to_py_err)?;
        }
        Ok(true)
    }
}

#[pymodule]
fn detris(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Environment>()?;
    Ok(())
}

// NOTE: https://pyo3.rs/v0.9.1/advanced.html#testing
#[cfg(disabled_test)]
mod test {
    use super::Environment;

    #[test]
    fn test() {
        let mut env = Environment::new();
        println!("{}", env.to_string());
        env.reset(None);
        println!("{}", env.to_string());
    }
}
