use super::Bot;
use deep_trinity_core::{Game, Placement, StatisticsEntryType, TSpin, LineClear, Statistics, MoveTransition};
use deep_trinity_grid::Grid;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::Action;
use std::error::Error;

const BUDGET: f32 = 5.0;
const CONSUMPTION_BY_HOLD: f32 = 0.0;
const MIN_CONSUMPTION_BY_MOVE: f32 = 1.0;
const CONSUMPTION_GROWTH_IN_MOVE: f32 = 0.0;

fn eval_state(game: &Game) -> f32 {
    let pf = &game.state.playfield;
    let n = pf.grid.num_covered_empty_cells() as f32;
    let threshold = pf.width() as f32 * pf.height() as f32 / 20.0;
    let r = if n > threshold {
        0.0
    } else {
        1.0 - n / threshold
    };
    r
}

fn calc_reward(diff: &Statistics) -> f32 {
    let mut reward = 0.0;
    for (ent_type, val) in &[
        (StatisticsEntryType::LineClear(LineClear::new(1, None)), 0.1),
        (StatisticsEntryType::LineClear(LineClear::new(2, None)), 0.2),
        (StatisticsEntryType::LineClear(LineClear::new(3, None)), 0.3),
        (StatisticsEntryType::LineClear(LineClear::new(4, None)), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Standard))), 1.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Standard))), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, Some(TSpin::Standard))), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Mini))), 0.2),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Mini))), 0.2),
        (StatisticsEntryType::PerfectClear, 5.0),
    ] {
        reward += diff.get(*ent_type) as f32 * val;
    }
    reward
}

fn eval_placement(p: &Placement) -> f32 {
    1.0 - (p.pos.1 + 5) as f32 / 50.0
}

#[derive(Debug)]
struct Node {
    _parent: Option<Weak<RefCell<Node>>>,
    children: HashMap<Action, Rc<RefCell<Node>>>,
    game: Game<'static>,
    reward: f32,
    max_future_reward: f32,
}

impl Node {
    fn new(game: Game<'static>, reward: f32, parent: Option<Weak<RefCell<Node>>>) -> Self {
        Self {
            _parent: parent,
            children: HashMap::new(),
            game,
            reward,
            max_future_reward: 0.0,
        }
    }
    fn max_reward(&self) -> f32 { self.reward + self.max_future_reward }
}

fn expand(rc_node: Rc<RefCell<Node>>, budget: f32) -> Result<(), Box<dyn Error>> {
    if budget <= 0.0 {
        return Ok(());
    }
    let mut remain = budget;
    let mut max_future_reward = 0.0;
    {
        let mut node = rc_node.borrow_mut();
        if node.game.state.can_hold {
            let mut next = node.game.clone();
            let ok = next.hold()?;
            let rc_child = Rc::new(RefCell::new(Node::new(next, 0.0, Some(Rc::downgrade(&rc_node)))));
            node.children.insert(Action::Hold, rc_child.clone());
            remain -= CONSUMPTION_BY_HOLD;
            if ok {
                expand(rc_child.clone(), remain)?;
            }
            max_future_reward = rc_child.borrow().max_reward();
        }
    }
    let candidates = rc_node.borrow().game.get_move_candidates()?;
    let mut children = candidates.iter()
        .map(|mt| {
            let (simulated, reward) = simulate(&rc_node.borrow().game, mt);
            let rc_child = Rc::new(RefCell::new(Node::new(simulated, reward, Some(Rc::downgrade(&rc_node)))));
            (mt, rc_child, reward)
        })
        .collect::<Vec<_>>();
    children.sort_by(|(_, _, r1), (_, _, r2)| r2.partial_cmp(&r1).unwrap());
    let mut consumption = MIN_CONSUMPTION_BY_MOVE;
    for (mt, rc_child, _) in children.iter() {
        remain -= consumption;
        if remain <= 0.0 {
            break;
        }
        rc_node.borrow_mut().children.insert(Action::Move((*mt).clone()), rc_child.clone());
        if rc_child.borrow().game.state.falling_piece.is_some() {
            expand(rc_child.clone(), remain)?;
        }
        let r = rc_child.borrow().max_reward();
        if max_future_reward < r {
            max_future_reward = r;
        }
        consumption += CONSUMPTION_GROWTH_IN_MOVE;
    }

    rc_node.borrow_mut().max_future_reward = max_future_reward;
    Ok(())
}

fn simulate(game: &Game<'static>, mt: &MoveTransition) -> (Game<'static>, f32) {
    let mut next_game = game.clone();
    let fp = next_game.state.falling_piece.as_mut().unwrap();
    if let Some(hint) = mt.hint {
        fp.placement = hint.placement;
        let ok = fp.apply_move(hint.by, &next_game.state.playfield, next_game.rules.rotation_mode);
        debug_assert!(ok);
    } else {
        fp.placement = mt.placement;
    }
    debug_assert_eq!(mt.placement, fp.placement);
    next_game.lock().unwrap();
    let stats_diff = next_game.stats.clone() - game.stats.clone();
    let reward =
        eval_placement(&mt.placement) * 0.2
            + calc_reward(&stats_diff) * 1.0
            + eval_state(&next_game) * 0.5;
    (next_game, reward)
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleTreeBot {}

impl Bot for SimpleTreeBot {
    fn think(&mut self, game: &Game<'static>) -> Result<Action, Box<dyn Error>> {
        let mut game = game.clone();
        game.state.next_pieces.remove_invisible();
        let node = Rc::new(RefCell::new(Node::new(game, 0.0, None)));
        expand(node.clone(), BUDGET)?;
        let action = node.borrow().children.iter()
            .max_by(|(_, n1), (_, n2)| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                n1.max_reward().partial_cmp(&n2.max_reward()).unwrap()
            })
            .map(|(a, _)| a.clone())
            .unwrap();
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleTreeBot;
    use crate::BotRunner;

    #[test]
    #[ignore]
    fn test_simple_bot2() {
        let seed = 0;
        let runner = BotRunner::new(10, true, Some(seed), false);
        let mut bot = SimpleTreeBot::default();
        let game = runner.run_with_no_hooks(&mut bot).unwrap();
        assert!(game.stats.lock > 5);
    }
}
