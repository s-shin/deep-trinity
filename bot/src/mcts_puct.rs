/// MCTS-base bot implementation by the PUCT algorithm.
/// https://doi.org/10.1007/978-3-642-40988-2_13
use crate::{Bot, Action};
use core::{Game, StatisticsEntryType, LineClear, TSpin, Grid};
use std::error::Error;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use rand::{Rng, SeedableRng};
use rand::seq::SliceRandom;
use rand::prelude::StdRng;

fn eval_game(game: &Game) -> f32 {
    let mut reward = 0.0;
    let pf = &game.state.playfield;
    {
        let height = pf.height() as f32;
        let top_padding = pf.grid.top_padding() as f32;
        reward += top_padding / height;
    }
    {
        let n = pf.grid.num_covered_empty_cells() as f32;
        let threshold = pf.width() as f32 * 2.0;
        reward += if n > threshold {
            0.0
        } else {
            1.0 - n / threshold
        };
    }
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
    0.5
}

fn exploration_coefficient(_depth: usize) -> f32 {
    0.01
}

// const NUM_ITERATIONS: usize = 5000;
const NUM_ITERATIONS: usize = 50;

#[derive(Clone, Debug)]
struct GameData {
    by: Action,
    game: Rc<Game>,
    actions: Vec<Action>,
}

impl GameData {
    fn new<R: Rng + ?Sized>(by: Action, game: Game, rng: &mut R) -> Result<Self, Box<dyn Error>> {
        let actions = if game.state.falling_piece.is_some() {
            let mut actions = game.get_move_candidates()?.iter()
                .map(|mt| { Action::Move(mt.clone()) })
                .collect::<Vec<_>>();
            if game.state.can_hold {
                actions.push(Action::Hold);
            }
            actions.shuffle(rng);
            actions
        } else {
            Vec::new()
        };
        Ok(Self { by, game: Rc::new(game), actions })
    }
}

#[derive(Debug)]
struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: Vec<Rc<RefCell<Node>>>,
    num_visits: usize,
    depth: usize,
    sum_value: f32,
    game_data: GameData,
}

impl Node {
    fn new(parent: Option<Weak<RefCell<Node>>>, depth: usize, game_data: GameData) -> Self {
        Self {
            parent,
            children: Vec::new(),
            num_visits: 0,
            depth,
            sum_value: 0.0,
            game_data,
        }
    }
    fn best_action(&self) -> Option<Action> {
        if self.children.is_empty() {
            return None;
        }
        let best = self.children.iter()
            .max_by(|n1, n2| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                n1.num_visits.cmp(&n2.num_visits)
            })
            .unwrap();
        Some(best.borrow().game_data.by)
    }
    #[allow(dead_code)]
    fn visit(&self, visitor: fn(node: &Node)) {
        visitor(self);
        for child in self.children.iter() {
            let child = child.borrow();
            child.visit(visitor);
        }
    }
    fn is_final(&self) -> bool {
        let s = &self.game_data.game.state;
        s.falling_piece.is_none() || s.is_game_over()
    }
    fn is_decision_node(&self) -> bool { self.depth % 2 == 0 }
}

fn iterate<R: Rng + ?Sized>(mut node: Rc<RefCell<Node>>, rng: &mut R) -> Result<(), Box<dyn Error>> {
    while !node.borrow().is_final() {
        node.borrow_mut().num_visits += 1;
        // {
        //     let node = node.borrow();
        //     println!(
        //         "!!! depth={}, num_visits={}, num_children={}, num_actions={}, fp={:?}",
        //         node.depth, node.num_visits, node.children.len(), node.game_data.actions.len(),
        //         node.game_data.game.state.falling_piece,
        //     );
        // }
        if node.borrow().is_decision_node() {
            let depth = node.borrow().depth;
            let alpha = progressive_widening_coefficient(depth);
            let n_z = node.borrow().num_visits as f32;
            if n_z.powf(alpha).floor() > (n_z - 1.0).powf(alpha).floor() {
                let (mut next_game, action) = {
                    let game_data = &mut node.borrow_mut().game_data;
                    let next_game = (*game_data.game).clone();
                    let action = game_data.actions.pop();
                    (next_game, action)
                };
                if action.is_none() {
                    continue;
                }
                let action = action.unwrap();
                match action {
                    Action::Move(mt) => {
                        let fp = next_game.state.falling_piece.as_mut().unwrap();
                        if let Some(hint) = mt.hint {
                            fp.placement = hint.placement;
                            let ok = fp.apply_move(hint.by, &next_game.state.playfield, next_game.rules.rotation_mode);
                            debug_assert!(ok);
                        } else {
                            fp.placement = mt.placement;
                        }
                        debug_assert_eq!(mt.placement, fp.placement);
                        next_game.lock()?;
                    }
                    Action::Hold => {
                        next_game.hold()?;
                    }
                };
                let next_game_data = GameData::new(action, next_game, rng)?;
                let child = Node::new(
                    Some(Rc::downgrade(&node)),
                    depth + 1,
                    next_game_data,
                );
                node.borrow_mut().children.push(Rc::new(RefCell::new(child)));
            } else {
                let e_d = exploration_coefficient(depth);
                debug_assert!(!node.borrow().children.is_empty());
                let child = node.borrow().children.iter()
                    .max_by(|n1, n2| {
                        let (n1, n2) = (n1.borrow(), n2.borrow());
                        let (n_za1, n_za2) = (n1.num_visits as f32, n2.num_visits as f32);
                        if n1.num_visits == 0 || n2.num_visits == 0 {
                            return n2.num_visits.cmp(&n1.num_visits);
                        }
                        let score1 = (n1.sum_value / n_za1) + (n_z.powf(e_d) / n_za1).sqrt();
                        let score2 = (n2.sum_value / n_za2) + (n_z.powf(e_d) / n_za2).sqrt();
                        score1.partial_cmp(&score2).unwrap()
                    })
                    .cloned()
                    .unwrap();
                node = child;
            }
        } else {
            let depth = node.borrow().depth;
            let alpha = progressive_widening_coefficient(depth);
            let n_w = node.borrow().num_visits as f32;
            if n_w.powf(alpha).floor() == (n_w - 1.0).powf(alpha).floor() {
                let child = node.borrow().children.iter()
                    .min_by(|n1, n2| {
                        let (n1, n2) = (n1.borrow(), n2.borrow());
                        n1.num_visits.cmp(&n2.num_visits)
                    })
                    .cloned()
                    .unwrap();
                node = child;
            } else {
                let child = Node::new(
                    Some(Rc::downgrade(&node)),
                    depth + 1,
                    node.borrow().game_data.clone(),
                );
                node.borrow_mut().children.push(Rc::new(RefCell::new(child)));
            }
        }
    }
    let value = eval_game(&node.borrow().game_data.game);
    node.borrow_mut().sum_value += value;
    while let Some(parent) = &node.clone().borrow().parent {
        match parent.upgrade() {
            Some(parent) => {
                parent.borrow_mut().sum_value += value;
                node = parent;
            }
            None => break,
        }
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
pub struct MctsPuctBot {}

impl Bot for MctsPuctBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let mut rng = StdRng::seed_from_u64(0);
        let mut game = game.clone();
        game.state.next_pieces.remove_invisible();
        let game_data = GameData::new(Action::Hold /* dummy */, game, &mut rng)?;
        let root = Rc::new(RefCell::new(
            Node::new(None, 0, game_data)
        ));
        for _ in 0..NUM_ITERATIONS {
            iterate(root.clone(), &mut rng)?;
        }
        // root.borrow().visit(|node| {
        //     println!(
        //         "{}{}|num_visits={}, sum_value={}, num_children={}",
        //         " ".repeat(node.depth * 2), node.depth, node.num_visits, node.sum_value,
        //         node.children.len(),
        //     );
        // });
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
        let game = test_bot(&mut bot, seed, 3, false).unwrap();
        assert!(game.stats.lock > 1);
    }
}
