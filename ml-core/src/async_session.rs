// WIP
use std::sync::{mpsc, Arc, Mutex};
use std::error::Error;
use crate::{GameSession, Action};

#[derive(Clone, Debug)]
enum AsyncRequest {
    Reset(Option<u64>),
    Step(Action),
}

pub struct AsyncGameSession {
    session_mtx: Arc<Mutex<GameSession>>,
    should_wait: bool,
    req_tx: Option<mpsc::Sender<AsyncRequest>>,
    res_rx: mpsc::Receiver<()>,
    handle: Option<std::thread::JoinHandle<Result<(), &'static str>>>,
}

impl AsyncGameSession {
    pub fn new() -> Result<Self, &'static str> {
        let session_mtx: Arc<Mutex<GameSession>> = Arc::new(Mutex::new(GameSession::new(None)?));
        let (req_tx, req_rx) = mpsc::channel();
        let (res_tx, res_rx) = mpsc::channel();

        let handle = {
            let session_mtx = session_mtx.clone();
            std::thread::spawn(move || {
                for req in req_rx {
                    match req {
                        AsyncRequest::Reset(rand_seed) => {
                            session_mtx.lock().as_mut().unwrap().reset(rand_seed)?;
                            res_tx.send(()).unwrap();
                        }
                        AsyncRequest::Step(action) => {
                            session_mtx.lock().as_mut().unwrap().step(action)?;
                            res_tx.send(()).unwrap();
                        }
                    }
                }
                Ok(())
            })
        };

        Ok(Self { session_mtx, should_wait: false, req_tx: Some(req_tx), res_rx, handle: Some(handle) })
    }
    pub fn reset(&mut self, rand_seed: Option<u64>) -> Result<(), &'static str> {
        assert!(!self.should_wait);
        self.should_wait = true;
        self.req_tx.as_ref().unwrap().send(AsyncRequest::Reset(rand_seed)).unwrap();
        Ok(())
    }
    pub fn step(&mut self, action: Action) -> Result<(), &'static str> {
        assert!(!self.should_wait);
        self.should_wait = true;
        self.req_tx.as_ref().unwrap().send(AsyncRequest::Step(action)).unwrap();
        Ok(())
    }
    pub fn should_wait(&self) -> bool { self.should_wait }
    pub fn wait(&self, timeout: Option<std::time::Duration>) -> Result<(), Box<dyn Error>> {
        if !self.should_wait {
            return Ok(());
        }
        if let Some(t) = timeout {
            self.res_rx.recv_timeout(t)?;
        } else {
            self.res_rx.recv()?;
        }
        Ok(())
    }
    pub fn stop_and_join(&mut self) -> Result<(), Box<dyn Error>> {
        self.req_tx.take();
        let handle = self.handle.take().unwrap();
        handle.join().unwrap()?;
        Ok(())
    }
    pub fn game_str(&self) -> String {
        assert!(!self.should_wait);
        self.session_mtx.lock().as_ref().unwrap().game_str()
    }
    pub fn legal_actions(&self) -> Vec<u32> {
        assert!(!self.should_wait);
        self.session_mtx.lock().as_ref().unwrap().legal_actions()
    }
    pub fn observation(&self) -> Vec<u32> {
        assert!(!self.should_wait);
        self.session_mtx.lock().as_ref().unwrap().observation()
    }
    pub fn last_reward(&self) -> f32 {
        assert!(!self.should_wait);
        self.session_mtx.lock().as_ref().unwrap().last_reward()
    }
    pub fn is_done(&self) -> bool {
        assert!(!self.should_wait);
        self.session_mtx.lock().as_ref().unwrap().is_done()
    }
}

//---

pub struct AsyncGameSessionMulti {
    sessions: Vec<AsyncGameSession>,
}

impl AsyncGameSessionMulti {
    pub fn new(n: usize) -> Result<Self, &'static str> {
        let mut sessions = Vec::with_capacity(n);
        for _ in 0..n {
            sessions.push(AsyncGameSession::new()?);
        }
        Ok(Self { sessions })
    }
    pub fn reset(&mut self) -> Result<(), &'static str> {
        for session in self.sessions.iter_mut() {
            session.reset(None)?;
        }
        Ok(())
    }
    pub fn step(&mut self, actions: Vec<Action>) -> Result<(), &'static str> {
        for (i, session) in self.sessions.iter_mut().enumerate() {
            session.step(actions[i])?;
        }
        Ok(())
    }
    pub fn should_wait(&self) -> bool {
        for session in self.sessions.iter() {
            if session.should_wait() {
                return true;
            }
        }
        return false;
    }
    pub fn wait(&self, timeout: Option<std::time::Duration>) -> Result<(), Box<dyn Error>> {
        for session in self.sessions.iter() {
            session.wait(timeout)?;
        }
        Ok(())
    }
    pub fn stop_and_join(&mut self) -> Result<(), Box<dyn Error>> {
        for session in self.sessions.iter_mut() {
            session.stop_and_join()?;
        }
        Ok(())
    }
    pub fn game_str(&self) -> Vec<String> {
        self.sessions.iter().map(|s| s.game_str()).collect::<_>()
    }
    pub fn legal_actions(&self) -> Vec<Vec<u32>> {
        self.sessions.iter().map(|s| s.legal_actions()).collect::<_>()
    }
    pub fn observation(&self) -> Vec<Vec<u32>> {
        self.sessions.iter().map(|s| s.observation()).collect::<_>()
    }
    pub fn last_reward(&self) -> Vec<f32> {
        self.sessions.iter().map(|s| s.last_reward()).collect::<_>()
    }
    pub fn is_done(&self) -> Vec<bool> {
        self.sessions.iter().map(|s| s.is_done()).collect::<_>()
    }
}

//---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_game_session_multi() {
        let mut agsm = AsyncGameSessionMulti::new(4).unwrap();
        agsm.reset().unwrap();
        agsm.wait(None).unwrap();
        for _ in 0..10 {
            let actions = agsm.legal_actions().iter().map(|actions| Action(actions[0])).collect::<Vec<_>>();
            agsm.step(actions).unwrap();
            agsm.wait(None).unwrap();
        }
        for s in agsm.game_str().iter() {
            println!("{}", s);
        }
        agsm.stop_and_join().unwrap();
    }
}
