use super::Bot;
use core::{Game, Placement, StatisticsEntryType, TSpin, LineClear, Statistics, Grid, FallingPiece};
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use std::collections::HashMap;

fn sigmoid(x: f32) -> f32 {
    ((x / 2.0).tanh() + 1.0) * 0.5
}

fn eval_state(game: &Game) -> f32 {
    // let pf = &game.state.playfield;
    // let fp = game.state.falling_piece.as_ref().unwrap();
    // let src_pos = core::UPos::from(fp.placement.pos);
    // let num_empty_cells = pf.width() as usize * pf.height() as usize -
    //     (pf.grid.num_blocks() + fp.grid().num_blocks());
    // if num_empty_cells == 0 {
    //     return 0.0;
    // }
    // pf.grid.bit_grid.enclosed_space(src_pos).len() as f32 / num_empty_cells as f32
    0.0
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
    sigmoid(reward)
}

fn eval_placement(p: &Placement) -> f32 {
    sigmoid(-p.pos.1 as f32 / 40.0)
}

fn eval_node_like_puct(parent: &Node, placement: &Option<Placement>, child: &Node) -> f32 {
    let q = child.reward;
    let p = if let Some(placement) = placement {
        let sum: f32 = parent.children.keys()
            .map(|p| p.map_or(0.0, |p| eval_placement(&p)))
            .sum();
        eval_placement(&placement) / sum
    } else {
        1.0 / parent.children.len() as f32
    };
    let c = 1000.0;
    q + c * p * (parent.num_simulated as f32).sqrt() / (1 + child.num_simulated) as f32
}

#[derive(Debug)]
struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: HashMap<Option<Placement>, Rc<RefCell<Node>>>,
    game: Game,
    reward: f32,
    num_simulated: usize,
}

impl Node {
    fn new(game: Game, reward: f32) -> Self {
        Self {
            parent: None,
            children: HashMap::new(),
            game,
            reward,
            num_simulated: 0,
        }
    }
    fn visit(&self, visitor: fn(node: &Node, depth: usize)) {
        fn visit_rec(node: &Node, visitor: fn(node: &Node, depth: usize), depth: usize) {
            visitor(node, depth);
            for child in node.children.values() {
                visit_rec(&child.borrow(), visitor, depth + 1);
            }
        }
        visit_rec(self, visitor, 0);
    }
}

fn select_child_node(rc_node: Rc<RefCell<Node>>) -> (Option<Placement>, Rc<RefCell<Node>>) {
    let node = rc_node.borrow();
    let r = node.children.iter()
        .max_by(|(p1, n1), (p2, n2)| {
            let v1 = eval_node_like_puct(&node, *p1, &n1.borrow());
            let v2 = eval_node_like_puct(&node, *p2, &n2.borrow());
            v1.partial_cmp(&v2).unwrap()
        })
        .unwrap();
    (r.0.clone(), r.1.clone())
}

fn select(rc_node: Rc<RefCell<Node>>, max_depth: usize) -> Option<Rc<RefCell<Node>>> {
    let mut current = rc_node;
    for _ in 0..max_depth {
        if current.borrow().children.is_empty() {
            return Some(current);
        }
        let next = select_child_node(current).1;
        current = next;
    }
    None
}

fn expand(rc_node: Rc<RefCell<Node>>) {
    let candidates = match rc_node.borrow().game.get_move_candidates() {
        Ok(r) => r,
        Err(_) => return,
    };
    if candidates.is_empty() {
        return;
    }
    for fp in candidates.iter() {
        let (simulated, reward) = simulate(&rc_node.borrow().game, fp);
        let mut child = Node::new(simulated, reward);
        child.parent = Some(Rc::downgrade(&rc_node));
        let rc_child = Rc::new(RefCell::new(child));
        rc_node.borrow_mut().children.insert(Some(fp.placement), rc_child.clone());
        backpropagate(rc_child);
    }
}

fn simulate(game: &Game, fp: &FallingPiece) -> (Game, f32) {
    let mut simulated = game.clone();
    simulated.state.falling_piece = Some(fp.clone());
    simulated.lock().unwrap();
    let stats_diff = simulated.stats.clone() - game.stats.clone();
    let reward =
        eval_placement(&fp.placement) * 0.1
            + calc_reward(&stats_diff) * 5.0
            + eval_state(&simulated) * 0.0;
    (simulated, reward)
}

fn backpropagate(rc_node: Rc<RefCell<Node>>) {
    let mut current = rc_node;
    let mut child_reward = 0.0;
    loop {
        {
            let mut current = current.borrow_mut();
            current.reward += child_reward;
            child_reward = current.reward;
            current.num_simulated += 1;
        }
        let next = if let Some(parent) = current.borrow().parent.clone() {
            if let Some(parent) = parent.upgrade() {
                parent
            } else {
                break;
            }
        } else {
            break;
        };
        current = next;
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct SimpleBot2 {}

impl Bot for SimpleBot2 {
    fn think(&mut self, game: &Game) -> Option<Placement> {
        let node = Rc::new(RefCell::new(Node::new(game.clone(), 0.0)));
        for _ in 0..500 {
            let selected = select(node.clone(), 5);
            if selected.is_none() {
                break;
            }
            expand(selected.unwrap());
        }
        // println!("---");
        // node.borrow().visit(|node, depth| {
        //     println!("{}simulated: {}, reward: {}", " ".repeat(depth), node.num_simulated, node.reward);
        // });
        // println!("---");
        select_child_node(node).0
    }
}

#[cfg(test)]
mod tests {
    use super::SimpleBot2;
    use crate::test_bot;

    #[test]
    fn test_simple_bot2() {
        let mut bot = SimpleBot2::default();
        let seed = 0;
        let game = test_bot(&mut bot, seed, 2, true);
        assert!(game.stats.lock > 40);
    }
}
