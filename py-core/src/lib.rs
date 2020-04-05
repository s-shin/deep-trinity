extern crate pyo3;
extern crate core;

use pyo3::prelude::*;
use ml_core::GameSession;

fn to_py_err(e: &'static str) -> PyErr {
    pyo3::exceptions::RuntimeError::py_err(e)
}

#[pyclass]
struct Environment {
    session: Option<GameSession>,
}

fn must_get_session(env: &Environment) -> PyResult<&GameSession> {
    if env.session.is_none() {
        return Err(to_py_err("game session is not initialized"));
    }
    Ok(env.session.as_ref().unwrap())
}

fn must_get_mut_session(env: &mut Environment) -> PyResult<&mut GameSession> {
    if env.session.is_none() {
        return Err(to_py_err("game session is not initialized"));
    }
    Ok(env.session.as_mut().unwrap())
}

#[pymethods]
impl Environment {
    #[new]
    fn new() -> Self {
        Self { session: None }
    }
    fn to_string(&self) -> String {
        if let Some(session) = self.session.as_ref() {
            session.game_str()
        } else {
            "".into()
        }
    }
    #[staticmethod]
    pub fn action_space() -> u32 { ml_core::NUM_ACTIONS }
    pub fn legal_actions(&self) -> PyResult<Vec<u32>> {
        Ok(must_get_session(self)?.legal_actions())
    }
    pub fn observation(&self) -> PyResult<Vec<u32>> {
        Ok(must_get_session(self)?.observation())
    }
    pub fn last_reward(&self) -> PyResult<f32> {
        Ok(must_get_session(self)?.last_reward())
    }
    pub fn is_done(&self) -> PyResult<bool> {
        Ok(must_get_session(self)?.is_done())
    }
    pub fn reset(&mut self, rand_seed: Option<u64>) -> PyResult<()> {
        self.session = Some(GameSession::new(rand_seed.unwrap_or(0)).map_err(to_py_err)?);
        Ok(())
    }
    pub fn step(&mut self, action_id: u32) -> PyResult<()> {
        must_get_mut_session(self)?.step(ml_core::Action(action_id)).map_err(to_py_err)
    }
}

#[pymodule]
fn core(_py: Python, m: &PyModule) -> PyResult<()> {
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
