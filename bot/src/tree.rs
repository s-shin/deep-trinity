use crate::{Bot, Action};
use core::{Game, BitGrid, MoveTransition, LineClear, Playfield, Piece, FallingPiece, PIECES, GameRules};
use std::error::Error;
use std::collections::{HashMap, HashSet};
use core::helper::get_move_candidates;
use std::cell::RefCell;
use std::rc::{Weak, Rc};

#[derive(Debug, Clone)]
struct MoveInfo {
    move_candidates: HashSet<MoveTransition>,
    line_clear_moves: HashMap<LineClear, MoveTransition>,
}

impl MoveInfo {
    fn collect(pf: &Playfield, piece: Piece, rules: &GameRules) -> Self {
        let move_candidates = get_move_candidates(pf, &FallingPiece::spawn(piece, Some(pf)), rules);
        let mut line_clear_moves = HashMap::new();
        for mt in move_candidates.iter() {
            let line_clear = pf.check_line_clear(
                &FallingPiece::new_with_last_move_transition(piece, mt),
                rules.tspin_judgement_mode);
            if line_clear.num_lines > 0 {
                line_clear_moves.insert(line_clear, *mt);
            }
        }
        Self {
            move_candidates,
            line_clear_moves,
        }
    }
}

#[derive(Debug, Clone)]
struct PlayfieldInfo {
    moves: Vec<MoveInfo>,
}

impl PlayfieldInfo {
    fn collect(pf: &Playfield, rules: &GameRules) -> Self {
        let moves = PIECES.iter()
            .map(|piece| MoveInfo::collect(pf, *piece, rules))
            .collect();
        Self { moves }
    }
    fn get_move_info(&self, piece: Piece) -> &MoveInfo {
        self.moves.get(piece.to_usize()).unwrap()
    }
}

struct PlayfieldMemory {
    rules: GameRules,
    memory: HashMap<BitGrid, PlayfieldInfo>,
}

impl PlayfieldMemory {
    fn new(rules: GameRules) -> Self {
        Self {
            rules,
            memory: HashMap::new(),
        }
    }
    fn register(&mut self, pf: &Playfield) {
        if !self.memory.contains_key(&pf.grid.bit_grid) {
            self.memory.insert(pf.grid.bit_grid.clone(), PlayfieldInfo::collect(pf, &self.rules));
        }
    }
    fn get(&self, bit_grid: &BitGrid) -> Option<&PlayfieldInfo> {
        self.memory.get(bit_grid)
    }
}

struct Node {
    parent: Option<Weak<RefCell<Node>>>,
    children: HashMap<Action, Rc<RefCell<Node>>>,
    game: Game,
}

impl Node {
    fn new(parent: Option<Weak<RefCell<Node>>>, game: Game) -> Self {
        Self {
            parent,
            children: HashMap::new(),
            game,
        }
    }
}

fn expand(rc_node: Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
    let mut node = rc_node.borrow_mut();
    if node.game.state.can_hold {
        let mut game = node.game.clone();
        let ok = game.hold()?;
        assert!(ok);
        let rc_child = Rc::new(RefCell::new(Node::new(Some(Rc::downgrade(&rc_node)), game)));
        node.children.insert(Action::Hold, rc_child);
    }
    let move_candidates = node.game.get_move_candidates()?;
    for mt in move_candidates.iter() {
        let mut game = node.game.clone();
        let piece = game.state.falling_piece.unwrap().piece;
        game.state.falling_piece = Some(FallingPiece::new_with_last_move_transition(piece, mt));
        game.lock()?;
        let rc_child = Rc::new(RefCell::new(Node::new(Some(Rc::downgrade(&rc_node)), game)));
        node.children.insert(Action::Move(*mt), rc_child);
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TreeBot {}

impl Bot for TreeBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let candidates = game.get_move_candidates()?;
        if candidates.is_empty() {
            return Err("no movable placements".into());
        }
        let selected = candidates.iter()
            .min_by(|mt1, mt2| mt1.placement.pos.1.cmp(&mt2.placement.pos.1))
            .unwrap();
        Ok(Action::Move(selected.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_bot;

    #[test]
    fn test_simple_bot() {
        let mut bot = TreeBot::default();
        let seed = 0;
        let game = test_bot(&mut bot, seed, 100, false).unwrap();
        assert!(game.stats.lock > 40);
    }
}
