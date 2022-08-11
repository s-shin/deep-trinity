mod core_wrapper;

use pyo3::prelude::*;
use ml_core::GameSession;

fn to_py_err(e: &'static str) -> PyErr {
    pyo3::exceptions::PyRuntimeError::new_err(e)
}

#[pyclass]
struct Environment {
    session: GameSession,
}

#[pymethods]
impl Environment {
    #[new]
    fn new() -> PyResult<Self> {
        let session = GameSession::new(None).map_err(to_py_err)?;
        Ok(Self { session })
    }
    fn clone(&self) -> Self {
        Self { session: self.session.clone() }
    }
    fn game_str(&self) -> String {
        self.session.game_str()
    }
    #[staticmethod]
    pub fn num_actions() -> u32 { ml_core::NUM_ACTIONS }
    pub fn legal_actions(&self) -> Vec<u32> { self.session.legal_actions() }
    pub fn observation(&self) -> Vec<u32> { self.session.observation() }
    pub fn last_reward(&self) -> f32 { self.session.last_reward() }
    pub fn is_done(&self) -> bool { self.session.is_done() }
    pub fn reset(&mut self, rand_seed: Option<u64>) -> PyResult<()> {
        self.session.reset(rand_seed).map_err(to_py_err)
    }
    pub fn step(&mut self, action_id: u32) -> PyResult<()> {
        self.session.step(ml_core::Action(action_id)).map_err(to_py_err)
    }
}

#[pymodule]
fn deep_trinity(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Environment>()?;
    m.add_class::<core_wrapper::CellWrapper>()?;
    m.add_class::<core_wrapper::PlacementWrapper>()?;
    m.add_class::<core_wrapper::MoveDecisionResourceWrapper>()?;
    m.add_class::<core_wrapper::GameWrapper>()?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::Environment;

    #[test]
    fn test() {
        Environment::new().unwrap();
    }
}
