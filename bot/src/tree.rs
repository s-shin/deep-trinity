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

fn expand_node(node: &Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
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

fn expand_leaves(node: &Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
    tree::visit(node, &mut (), |node, _, _| {
        if !node.borrow().is_leaf() {
            return tree::VisitPlan::Children;
        }
        expand_node(node).unwrap_or_default();
        tree::VisitPlan::Sibling
    });
    Ok(())
}

#[derive(Copy, Clone, Debug, Default)]
pub struct TreeBot {}

impl Bot for TreeBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let root = tree::new(NodeData::new(None, game.clone()));
        expand_leaves(&root)?;
        expand_leaves(&root)?;

        let paths = tree::get_all_paths_to_leaves(&root);

        // TODO: filter line clear?

        let (_, paths) = paths.iter()
            .fold((-1, vec![]), |(min, mut paths), path| {
                const FACTOR: i32 = 10;
                let n = path.child_node_iter(&root)
                    .enumerate()
                    .fold(0, |acc, (i, node)| {
                        acc + (node.borrow().data.num_covered_empty_cells * i) as i32 * FACTOR
                    });
                if min == -1 || n < min {
                    (n, vec![path])
                } else if n == min {
                    paths.push(path);
                    (n, paths)
                } else {
                    (min, paths)
                }
            });

        let paths = {
            const TRENCH_THRESHOLD: i8 = 3;
            const MAX_TRENCH: u8 = 1;
            let ps = paths.iter()
                .filter(|path| {
                    let node = tree::get(&root, path.iter()).unwrap();
                    let contour = node.borrow().data.game.state.playfield.grid.contour();
                    let mut n = 0;
                    for i in 0..(contour.len() - 1) {
                        if (contour[i] as i8 - contour[i + 1] as i8).abs() >= TRENCH_THRESHOLD {
                            n += 1;
                            if n >= MAX_TRENCH {
                                return false;
                            }
                        }
                    }
                    true
                })
                .copied()
                .collect::<Vec<_>>();
            if ps.is_empty() {
                paths
            } else {
                ps
            }
        };

        let (_, paths) = paths.iter()
            .fold((-1, vec![]), |(min, mut paths), &path| {
                let n = tree::ChildNodeIterator::new(&root, path.iter())
                    .filter(|node| matches!(node.borrow().data.by, Some(Action::Hold)))
                    .count() as i32;
                if min == -1 || n < min {
                    (n, vec![path])
                } else if n == min {
                    paths.push(path);
                    (n, paths)
                } else {
                    (min, paths)
                }
            });

        let (_, path) = paths.iter()
            .map(|&path| {
                let node = tree::get(&root, path.iter()).unwrap();
                let dencity = node.borrow().data.game.state.playfield.grid.density_without_top_padding();
                ((dencity * 10000.0) as u32, path)
            })
            .max()
            .unwrap();

        let action = tree::get(&root, [path.indices[0]].iter()).unwrap().borrow().data.by.unwrap();
        Ok(action)

        // #[derive(Debug)]
        // struct Context {
        //     paths: Vec<tree::Path>,
        //     min_num_covered_empty_cells: usize,
        // }
        // let mut ctx = Context {
        //     paths: vec![],
        //     min_num_covered_empty_cells: 1000000,
        // };
        //
        // tree::visit(&root, &mut ctx, |node, ctx, state| {
        //     // println!("{}{:?}", " ".repeat(state.path.len() * 2), node.data.by);
        //     let node = node.borrow();
        //     if node.is_root() || !node.is_leaf() || matches!(node.data.by, Some(Action::Hold)) {
        //         return tree::VisitPlan::Children;
        //     }
        //     if ctx.min_num_covered_empty_cells >= node.data.num_covered_empty_cells {
        //         if ctx.min_num_covered_empty_cells > node.data.num_covered_empty_cells {
        //             ctx.paths.clear();
        //         }
        //         ctx.paths.push(state.path.clone());
        //         ctx.min_num_covered_empty_cells = node.data.num_covered_empty_cells;
        //     }
        //     tree::VisitPlan::Children
        // });
        //
        // println!("ctx: {:?}", ctx);
        //
        // // Exclude hold actions.
        // let mut paths = ctx.paths.iter()
        //     .filter(|path| {
        //         path.child_node_iter(&root)
        //             .find(|node| matches!(node.borrow().data.by, Some(Action::Hold)))
        //             .is_none()
        //     })
        //     .collect::<Vec<_>>();
        // if paths.is_empty() {
        //     paths = ctx.paths.iter().collect();
        // }
        //
        // // TODO: filter line clear?
        // // TODO: escape multiple trenches.
        //
        // // Prefer higher density and less hold actions.
        // let path = paths.iter()
        //     .map(|path| {
        //         let num_holds = tree::ChildNodeIterator::new(&root, path.iter())
        //             .fold(0, |acc, node| {
        //                 if matches!(node.borrow().data.by, Some(Action::Hold)) {
        //                     acc + 1
        //                 } else {
        //                     acc
        //                 }
        //             });
        //         let leaf = tree::get(&root, path.iter()).unwrap();
        //         let density = &leaf.borrow().data.game.state.playfield.grid.density_without_top_padding();
        //         ((100 - (density * 100.0) as u8), num_holds, path)
        //     })
        //     .min();
        // if let Some((_, _, path)) = path {
        //     let action = tree::get(&root, [path[0]].iter()).unwrap().borrow().data.by.unwrap();
        //     return Ok(action);
        // }
        //
        // let action = root.borrow().children.get(0).unwrap().borrow().data.by.unwrap();
        // Ok(action)
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
        let game = test_bot(&mut bot, seed, 40, true).unwrap();
        assert!(game.stats.lock > 40);
    }
}
