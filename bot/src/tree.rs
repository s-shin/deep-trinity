use crate::{Bot, Action};
use core::{Game, FallingPiece, Grid};
use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct NodeData {
    by: Option<Action>,
    game: Game,
    num_covered_empty_cells: usize,
}

impl NodeData {
    fn new(by: Option<Action>, game: Game) -> Self {
        let num_covered_empty_cells = game.state.playfield.grid.num_covered_empty_cells();
        Self {
            by,
            game,
            num_covered_empty_cells,
        }
    }
}

type Node = tree::Node<NodeData>;

fn expand(node: &Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
    if node.borrow().data.game.state.can_hold {
        let mut game = node.borrow().data.game.clone();
        let ok = game.hold()?;
        assert!(ok);
        let data = NodeData::new(Some(Action::Hold), game);
        tree::append_child(node, data);
    }
    let move_candidates = node.borrow().data.game.get_move_candidates()?;
    for mt in move_candidates.iter() {
        let mut game = node.borrow().data.game.clone();
        let piece = game.state.falling_piece.unwrap().piece;
        game.state.falling_piece = Some(FallingPiece::new_with_last_move_transition(piece, mt));
        game.lock()?;
        let data = NodeData::new(Some(Action::Move(*mt)), game);
        tree::append_child(node, data);
    }
    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TreeBot {}

impl Bot for TreeBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let root = tree::new(NodeData::new(None, game.clone()));
        expand(&root)?;

        struct Context {
            paths: Vec<Vec<usize>>,
            min_num_covered_empty_cells: usize,
        }
        let mut ctx = Context {
            paths: vec![],
            min_num_covered_empty_cells: 1000000,
        };

        tree::visit(&root, &mut ctx, |node, ctx, state| {
            if node.is_root() || matches!(node.data.by, Some(Action::Hold)) {
                return tree::VisitPlan::Children;
            }
            if ctx.min_num_covered_empty_cells >= node.data.num_covered_empty_cells {
                if ctx.min_num_covered_empty_cells > node.data.num_covered_empty_cells {
                    ctx.paths.clear();
                }
                ctx.paths.push(state.path.clone());
                ctx.min_num_covered_empty_cells = node.data.num_covered_empty_cells;
            }
            if node.data.num_covered_empty_cells == 0 {}
            // println!("{}{:?}", " ".repeat(state.path.len() * 2), node.data.by);
            tree::VisitPlan::Children
        });

        if let Some(path) = ctx.paths.get(0) {
            let action = tree::get(&root, [path[0]].iter()).unwrap().borrow().data.by.unwrap();
            return Ok(action);
        }

        let action = root.borrow().children.get(0).unwrap().borrow().data.by.unwrap();
        Ok(action)
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
        let game = test_bot(&mut bot, seed, 10, true).unwrap();
        assert!(game.stats.lock > 40);
    }
}
