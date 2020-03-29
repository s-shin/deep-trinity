/// MCTS-base bot implementation by the PUCT algorithm.
/// https://doi.org/10.1007/978-3-642-40988-2_13
use crate::{Bot, Action};
use core::{Game, FallingPiece};
use std::error::Error;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;
use rand::{thread_rng, Rng};
use rand::seq::SliceRandom;

fn progressive_widening_coefficient(depth: usize) -> f32 {
    1.0
}

fn exploration_coefficient(depth: usize) -> f32 {
    1.0
}

type Reward = f32;

struct GameData {
    game: Game,
    actions: Vec<Action>,
}

impl GameData {
    fn new<R: Rng + ?Sized>(game: Game, rng: &mut R) -> Result<Self, Box<dyn Error>> {
        let mut actions = game.get_move_candidates()?.iter()
            .map(|fp| { Action::Move(fp.placement) })
            .collect::<Vec<_>>();
        if game.state.can_hold {
            actions.push(Action::Hold);
        }
        actions.shuffle(rng);
        Ok(Self { game, actions })
    }
}

struct DecisionNode {
    parent: Option<Weak<RefCell<RandomNode>>>,
    children: HashMap<Action, Rc<RefCell<RandomNode>>>,
    num_visits: usize,
    depth: usize,
    game_data: Rc<RefCell<GameData>>,
}

impl DecisionNode {
    fn new(parent: Option<Weak<RefCell<RandomNode>>>, depth: usize, game_data: Rc<RefCell<GameData>>) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            num_visits: 0,
            depth,
            game_data,
        }
    }
}

trait DecisionNodeMethods {
    fn select(&mut self) -> Result<(), Box<dyn Error>>;
    fn expand(&mut self) -> Result<bool, Box<dyn Error>>;
}

impl DecisionNodeMethods for Rc<RefCell<DecisionNode>> {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        self.borrow_mut().num_visits += 1;
        let depth = self.borrow().depth;
        let alpha = progressive_widening_coefficient(depth);

        loop {
            let n_z = self.borrow().num_visits as f32;
            if n_z.powf(alpha).abs() <= (n_z - 1.0).powf(alpha).abs() {
                break;
            }
            if !self.expand()? {
                break;
            }
            self.borrow_mut().num_visits += 1;
        }

        let children = &mut self.borrow_mut().children;
        if children.is_empty() {
            return Ok(()); // TODO: correct?
        }
        let e_d = exploration_coefficient(depth);
        let n_z = self.borrow().num_visits as f32;
        let selected = children.iter_mut()
            .max_by(|(_, n1), (_, n2)| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                let score1 = n1.avg_reward + (n_z.powf(e_d) / n1.num_visits as f32).sqrt();
                let score2 = n2.avg_reward + (n_z.powf(e_d) / n2.num_visits as f32).sqrt();
                score1.partial_cmp(&score2).unwrap()
            })
            .unwrap();
        selected.1.select()
    }
    fn expand(&mut self) -> Result<bool, Box<dyn Error>> {
        // let mut game_data = self.borrow_mut().game_data.borrow_mut();
        // let mut game = game_data.game.clone();
        // match game_data.actions.pop() {
        //     None => Ok(false),
        //     Some(Action::MoveTo(dst)) => panic!(),
        //     Some(Action::Hold) => game.hold()
        // }
        // game.state.falling_piece = Some(fp);
        panic!("TODO")
    }
}

struct RandomNode {
    parent: Weak<RefCell<DecisionNode>>,
    children: Vec<Rc<RefCell<DecisionNode>>>,
    num_visits: usize,
    depth: usize,
    avg_reward: Reward,
    game_data: Rc<RefCell<GameData>>,
}

impl RandomNode {
    fn new(parent: Weak<RefCell<DecisionNode>>, depth: usize, game_data: Rc<RefCell<GameData>>) -> Self {
        Self {
            parent,
            children: Vec::new(),
            num_visits: 0,
            depth,
            avg_reward: 0.0,
            game_data,
        }
    }
}

trait RandomNodeMethods {
    fn select(&mut self) -> Result<(), Box<dyn Error>>;
}

impl RandomNodeMethods for Rc<RefCell<RandomNode>> {
    fn select(&mut self) -> Result<(), Box<dyn Error>> {
        self.borrow_mut().num_visits += 1;
        let depth = self.borrow().depth;
        let alpha = progressive_widening_coefficient(depth);

        loop {
            let n_w = self.borrow().num_visits as f32;
            if n_w.powf(alpha).abs() > (n_w - 1.0).powf(alpha).abs() {
                break;
            }
            let node = DecisionNode::new(
                Some(Rc::downgrade(&self)), depth + 1, self.borrow().game_data.clone());
            self.borrow_mut().children.push(Rc::new(RefCell::new(node)));
            self.borrow_mut().num_visits += 1;
        }

        let children = &mut self.borrow_mut().children;
        if children.is_empty() {
            return Ok(()); // TODO: correct?
        }
        let selected = children.iter_mut()
            .min_by(|n1, n2| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                n1.num_visits.cmp(&n2.num_visits)
            })
            .unwrap();
        selected.select()
    }
}

struct PuctBot {}

impl Bot for PuctBot {
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

