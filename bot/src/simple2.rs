use super::Bot;
use core::{Game, Placement, StatisticsEntryType, TSpin, LineClear, Statistics, FallingPiece, Grid};
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;
use crate::Action;
use std::error::Error;

const MAX_DEPTH: usize = 4;
const CUTOFF: usize = 5;

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
        (StatisticsEntryType::LineClear(LineClear::new(1, None)), 1.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, None)), 3.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, None)), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(4, None)), 8.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Standard))), 5.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Standard))), 8.0),
        (StatisticsEntryType::LineClear(LineClear::new(3, Some(TSpin::Standard))), 9.0),
        (StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Mini))), 2.0),
        (StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Mini))), 4.0),
        (StatisticsEntryType::PerfectClear, 10.0),
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
    parent: Option<Weak<RefCell<Node>>>,
    children: HashMap<Option<Placement>, Rc<RefCell<Node>>>,
    game: Game,
    reward: f32,
    max_future_reward: f32,
}

impl Node {
    fn new(game: Game, reward: f32) -> Self {
        Self {
            parent: None,
            children: HashMap::new(),
            game,
            reward,
            max_future_reward: 0.0,
        }
    }
}

fn expand(rc_node: Rc<RefCell<Node>>, max_depth: usize) -> Result<bool, Box<dyn Error>> {
    if max_depth == 0 {
        return Ok(false);
    }
    let candidates = rc_node.borrow().game.get_move_candidates()?;
    if candidates.is_empty() {
        return Ok(false);
    }
    let mut children = candidates.iter()
        .map(|fp| {
            let (simulated, reward) = simulate(&rc_node.borrow().game, fp);
            let mut child = Node::new(simulated, reward);
            child.parent = Some(Rc::downgrade(&rc_node));
            (fp, Rc::new(RefCell::new(child)), reward)
        })
        .collect::<Vec<_>>();
    children.sort_by(|(_, _, r1), (_, _, r2)| r2.partial_cmp(&r1).unwrap());
    let children = children.iter().take(CUTOFF).collect::<Vec<_>>();
    let mut max_future_reward = 0.0;
    for (fp, rc_child, _) in children.iter() {
        rc_node.borrow_mut().children.insert(Some(fp.placement), rc_child.clone());

        expand(rc_child.clone(), max_depth - 1)?;

        let child = rc_child.borrow();
        let r = child.reward + child.max_future_reward;
        if max_future_reward < r {
            max_future_reward = r;
        }
    }

    rc_node.borrow_mut().max_future_reward = max_future_reward;
    Ok(true)
}

fn simulate(game: &Game, fp: &FallingPiece) -> (Game, f32) {
    let mut simulated = game.clone();
    simulated.state.falling_piece = Some(fp.clone());
    simulated.lock().unwrap();
    let stats_diff = simulated.stats.clone() - game.stats.clone();
    let reward =
        eval_placement(&fp.placement) * 0.1
            + calc_reward(&stats_diff) * 0.1
            + eval_state(&simulated) * 1.0;
    (simulated, reward)
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot2 {}

impl Bot for SimpleBot2 {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let node = Rc::new(RefCell::new(Node::new(game.clone(), 0.0)));
        expand(node.clone(), MAX_DEPTH)?;
        let node = node.borrow();
        let dst = node.children.iter()
            .max_by(|(_, n1), (_, n2)| {
                let n1 = n1.borrow();
                let n2 = n2.borrow();
                (n1.reward + n1.max_future_reward).partial_cmp(&(n2.reward + n2.max_future_reward)).unwrap()
            })
            .map(|(p, _)| p.clone())
            .unwrap();
        dst.map_or(Err("no movable placements".into()), |p| Ok(Action::MoveTo(p)))
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleBot2;
    use crate::test_bot;

    #[test]
    #[ignore]
    fn test_simple_bot2() {
        let mut bot = SimpleBot2::default();
        let seed = 0;
        let game = test_bot(&mut bot, seed, 10, true).unwrap();
        assert!(game.stats.lock > 5);
    }
}
