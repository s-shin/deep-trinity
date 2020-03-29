/// MCTS-base bot implementation by the PUCT algorithm.
/// https://doi.org/10.1007/978-3-642-40988-2_13
use crate::{Bot, Action};
use core::{Game, StatisticsEntryType, LineClear, TSpin};
use std::error::Error;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

type Reward = f32;

fn eval_game(game: &Game) -> Reward {
    let mut reward = 0.0;
    for (ent_type, val) in &[
        (StatisticsEntryType::LineClear(LineClear::new(1, None)), 0.5),
        (StatisticsEntryType::LineClear(LineClear::new(2, None)), 1.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, None)), 3.0),
        (StatisticsEntryType::LineClear(LineClear::new(4, None)), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Standard))), 3.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Standard))), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, Some(TSpin::Standard))), 6.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Mini))), 1.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Mini))), 2.0),
        (StatisticsEntryType::PerfectClear, 10.0),
    ] {
        reward += game.stats.get(*ent_type) as f32 * val;
    }
    reward
}

fn progressive_widening_coefficient(_depth: usize) -> f32 {
    1.0
}

fn exploration_coefficient(_depth: usize) -> f32 {
    1.0
}

#[derive(Debug)]
struct GameData {
    game: Game,
    actions: Vec<Action>,
}

impl GameData {
    fn new<R: Rng + ?Sized>(game: Game, rng: &mut R) -> Result<Self, Box<dyn Error>> {
        let mut actions = game.get_move_candidates()?.iter()
            .map(|mt| { Action::Move(mt.clone()) })
            .collect::<Vec<_>>();
        if game.state.can_hold {
            actions.push(Action::Hold);
        }
        actions.shuffle(rng);
        Ok(Self { game, actions })
    }
}

#[derive(Debug)]
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
            num_visits: 1,
            depth,
            game_data,
        }
    }
    fn best_action(&self) -> Option<Action> {
        if self.children.is_empty() {
            return None;
        }
        let best = self.children.iter()
            .max_by(|(_, n1), (_, n2)| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                n1.num_visits.cmp(&n2.num_visits)
            })
            .unwrap();
        Some(best.0.clone())
    }
    fn backpropagate(&mut self, reward: Reward) -> Result<(), Box<dyn Error>> {
        if let Some(Some(parent)) = self.parent.as_ref().map(|p| p.upgrade()) {
            parent.borrow_mut().backpropagate(reward)?;
        }
        Ok(())
    }
}

trait DecisionNodeMethods {
    fn iterate(&mut self) -> Result<(), Box<dyn Error>>;
    fn select(&mut self) -> Result<(), Box<dyn Error>>;
    fn expand(&mut self) -> Result<bool, Box<dyn Error>>;
}

impl DecisionNodeMethods for Rc<RefCell<DecisionNode>> {
    fn iterate(&mut self) -> Result<(), Box<dyn Error>> {
        self.select()
    }
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

        if self.borrow().children.is_empty() {
            let reward = eval_game(&self.borrow().game_data.borrow().game);
            self.borrow_mut().backpropagate(reward)?;
            return Ok(());
        }

        let e_d = exploration_coefficient(depth);
        let n_z = self.borrow().num_visits as f32;
        let mut selected = self.borrow_mut().children.values()
            .max_by(|n1, n2| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                let score1 = n1.sum_reward / n1.num_visits as f32 + (n_z.powf(e_d) / n1.num_visits as f32).sqrt();
                let score2 = n2.sum_reward / n2.num_visits as f32 + (n_z.powf(e_d) / n2.num_visits as f32).sqrt();
                score1.partial_cmp(&score2).unwrap_or(std::cmp::Ordering::Equal)
            })
            .cloned()
            .unwrap();
        selected.select()
    }
    fn expand(&mut self) -> Result<bool, Box<dyn Error>> {
        // TODO: debug
        let rc_game_data = self.borrow_mut().game_data.clone();
        let mut game_data = rc_game_data.borrow_mut();
        let mut game = game_data.game.clone();
        let action = game_data.actions.pop();
        if action.is_none() {
            return Ok(false);
        }
        let action = action.unwrap();
        match action {
            Action::Move(mt) => {
                let fp = game.state.falling_piece.as_mut().unwrap();
                fp.placement = mt.src;
                let ok = fp.apply_move(mt.by, &game.state.playfield, game.rules.rotation_mode);
                debug_assert!(ok);
                debug_assert_eq!(mt.dst, fp.placement);
                game.lock()?;
            }
            Action::Hold => { game.hold()?; }
        }
        let depth = self.borrow().depth;
        let mut rng = thread_rng();
        let node = RandomNode::new(
            Rc::downgrade(&self),
            depth + 1,
            Rc::new(RefCell::new(GameData::new(game, &mut rng)?)));
        self.borrow_mut().children.insert(action, Rc::new(RefCell::new(node)));
        Ok(true)
    }
}

#[derive(Debug)]
struct RandomNode {
    parent: Weak<RefCell<DecisionNode>>,
    children: Vec<Rc<RefCell<DecisionNode>>>,
    num_visits: usize,
    depth: usize,
    sum_reward: Reward,
    game_data: Rc<RefCell<GameData>>,
}

impl RandomNode {
    fn new(parent: Weak<RefCell<DecisionNode>>, depth: usize, game_data: Rc<RefCell<GameData>>) -> Self {
        Self {
            parent,
            children: Vec::new(),
            num_visits: 1,
            depth,
            sum_reward: 0.0,
            game_data,
        }
    }
    fn backpropagate(&mut self, reward: Reward) -> Result<(), Box<dyn Error>> {
        self.sum_reward += reward;
        if let Some(parent) = self.parent.upgrade().as_mut() {
            parent.borrow_mut().backpropagate(reward)?;
        }
        Ok(())
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

        if self.borrow().children.is_empty() {
            let reward = eval_game(&self.borrow().game_data.borrow().game);
            self.borrow_mut().backpropagate(reward)?;
            return Ok(());
        }

        let mut selected = self.borrow_mut().children.iter_mut()
            .min_by(|n1, n2| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                n1.num_visits.cmp(&n2.num_visits)
            })
            .cloned()
            .unwrap();
        selected.select()
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MctsPuctBot {}

impl Bot for MctsPuctBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let mut rng = thread_rng();
        let game_data = GameData::new(game.clone(), &mut rng)?;
        let mut root = Rc::new(RefCell::new(
            DecisionNode::new(None, 0, Rc::new(RefCell::new(game_data)))
        ));
        for _ in 0..100 {
            root.iterate()?;
        }
        let action = root.borrow().best_action();
        Ok(action.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bot;

    #[test]
    fn test_simple_bot2() {
        let mut bot = MctsPuctBot::default();
        let seed = 0;
        let game = test_bot(&mut bot, seed, 10, true).unwrap();
        assert!(game.stats.lock > 5);
    }
}
