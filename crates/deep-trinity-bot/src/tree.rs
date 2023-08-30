use crate::{Bot, Action};
use deep_trinity_core::{Game, FallingPiece, Piece, LineClear};
use deep_trinity_grid::Grid;
use std::error::Error;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct NodeData {
    by: Option<Action>,
    game: Game<'static>,
    num_covered_empty_cells: usize,
    stop: bool,
}

impl NodeData {
    fn new(by: Option<Action>, game: Game<'static>, stop: bool) -> Self {
        let num_covered_empty_cells = game.state.playfield.grid.num_covered_empty_cells();
        Self {
            by,
            game,
            num_covered_empty_cells,
            stop,
        }
    }
}

type Node = deep_trinity_tree::Node<NodeData>;

fn expand_node(node: &Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
    if node.borrow().data.game.state.can_hold {
        let mut game = node.borrow().data.game.clone();
        game.stats = Default::default();
        let ok = game.hold()?;
        assert!(ok);
        let data = NodeData::new(Some(Action::Hold), game, false);
        deep_trinity_tree::append_child(node, data);
    }
    let move_candidates = node.borrow().data.game.get_move_candidates()?;
    for mt in move_candidates.iter() {
        let mut game = node.borrow().data.game.clone();
        game.stats = Default::default();
        let piece_spec = game.state.falling_piece.unwrap().piece_spec;
        game.state.falling_piece = Some(FallingPiece::new_with_last_move_transition(piece_spec, mt));
        game.lock()?;
        let data = NodeData::new(Some(Action::Move(*mt)), game, false);
        deep_trinity_tree::append_child(node, data);
    }
    Ok(())
}

fn expand_leaves(node: &Rc<RefCell<Node>>) -> Result<(), Box<dyn Error>> {
    deep_trinity_tree::visit(node, |node, _| {
        if !node.borrow().is_leaf() {
            return deep_trinity_tree::VisitPlan::Children;
        }
        if !node.borrow().data.stop {
            expand_node(node).unwrap_or_default();
        }
        deep_trinity_tree::VisitPlan::Sibling
    });
    Ok(())
}

// fn expand_leaves_mt(node: &Rc<RefCell<Node>>, n) -> Result<(), Box<dyn Error>> {
// }

//---

// fn evaluate_flatness(game: &Game) {
//     //
// }

//---

trait Filter {
    fn filter<'a>(&mut self, root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path>;
}

fn to_box_filter<F: Filter + 'static>(f: F) -> Box<dyn Filter> { Box::new(f) }

// NOTE: for<'a> syntax is called 'higher-ranked trait bound'.
type FilterFunc = for<'a> fn(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path>;

struct FunctionFilter(FilterFunc);

impl Filter for FunctionFilter {
    fn filter<'a>(&mut self, root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
        self.0(root, paths)
    }
}

fn to_box_filter_vec(fs: &[FilterFunc]) -> Vec<Box<dyn Filter>> {
    fs.iter().map(|f| Box::new(FunctionFilter(*f)) as Box<dyn Filter>).collect::<Vec<_>>()
}

struct OrFilter {
    filters: Vec<Box<dyn Filter>>,
}

impl OrFilter {
    fn new(filters: Vec<Box<dyn Filter>>) -> Self {
        Self { filters }
    }
}

impl Filter for OrFilter {
    fn filter<'a>(&mut self, root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
        for f in self.filters.iter_mut() {
            let r = f.filter(root, paths);
            if !r.is_empty() {
                return r;
            }
        }
        vec![]
    }
}

struct FilterChain {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    fn new(filters: Vec<Box<dyn Filter>>) -> Self {
        Self { filters }
    }
}

impl Filter for FilterChain {
    fn filter<'a>(&mut self, root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
        let mut paths = paths.to_vec();
        for f in self.filters.iter_mut() {
            let r = f.filter(root, &paths);
            if !r.is_empty() {
                paths = r;
            }
            if paths.len() == 1 {
                break;
            }
        }
        paths
    }
}

//---

fn exclude_stopped<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    paths.iter()
        .filter(|path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let stop = node.borrow().data.stop;
            !stop
        })
        .copied()
        .collect::<Vec<_>>()
}

fn min_covered_empty_cells<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    paths.iter()
        .fold((-1, vec![]), |(min, mut paths), &path| {
            const FACTOR: i32 = 10;
            let n = path.child_node_iter(&root)
                .enumerate()
                .fold(0, |acc, (i, node)| {
                    acc + node.borrow().data.num_covered_empty_cells as i32 * (FACTOR << i)
                });
            if min == -1 || n < min {
                (n, vec![path])
            } else if n == min {
                paths.push(path);
                (n, paths)
            } else {
                (min, paths)
            }
        }).1
}

fn hold_i<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    let state = &root.borrow().data.game.state;
    let piece = state.falling_piece.as_ref().unwrap().piece_spec.piece;
    if !state.can_hold || piece != Piece::I || matches!(state.hold_piece, Some(Piece::I)) {
        return vec![];
    }
    paths.iter()
        .filter(|path| {
            if let Some(node) = path.child_node_iter(root).next() {
                matches!(node.borrow().data.by, Some(Action::Hold))
            } else {
                false
            }
        })
        .copied()
        .collect::<Vec<_>>()
}

fn tetris<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    paths.iter()
        .filter(|path| {
            path.child_node_iter(root)
                .find(|node| {
                    node.borrow().data.game.stats.line_clear.get(&LineClear::tetris()) > 0
                })
                .is_some()
        })
        .copied()
        .collect::<Vec<_>>()
}

struct SuppressLineClear {
    height: deep_trinity_grid::Y,
}

impl SuppressLineClear {
    fn new(height: deep_trinity_grid::Y) -> Self {
        Self { height }
    }
}

impl Filter for SuppressLineClear {
    fn filter<'a>(&mut self, root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
        let h = {
            let game = &root.borrow().data.game;
            let grid = &game.state.playfield.grid;
            grid.height() - grid.top_padding()
        };
        if h > self.height {
            paths.to_vec()
        } else {
            paths.iter()
                .filter(|path| {
                    if let Some(node) = path.child_node_iter(root).next() {
                        let lcs = &node.borrow().data.game.stats.line_clear;
                        lcs.get(&LineClear::new(1, None)) == 0
                            && lcs.get(&LineClear::new(2, None)) == 0
                            && lcs.get(&LineClear::new(3, None)) == 0
                    } else {
                        false
                    }
                })
                .copied()
                .collect::<Vec<_>>()
        }
    }
}

fn _exclude_holds<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    paths.iter()
        .filter(|path| {
            path.child_node_iter(root)
                .find(|node| matches!(node.borrow().data.by, Some(Action::Hold)))
                .is_none()
        })
        .copied()
        .collect()
}

fn _exclude_trenches<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    const TRENCH_HEIGHT: i8 = 3;
    const MAX_TRENCHES: i8 = 2;
    paths.iter()
        .filter(|path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let hs = node.borrow().data.game.state.playfield.grid.contour();
            let mut n = 0;
            for i in 0..hs.len() {
                let left = if i == 0 { true } else { (hs[i] as i8 - hs[i - 1] as i8).abs() >= TRENCH_HEIGHT };
                let right = if i == hs.len() - 1 { true } else { (hs[i] as i8 - hs[i + 1] as i8).abs() >= TRENCH_HEIGHT };
                if left && right {
                    n += 1;
                    if n > MAX_TRENCHES {
                        return false;
                    }
                }
            }
            true
        })
        .copied()
        .collect()
}

fn _min_trenches<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    const TRENCH_HEIGHT: i8 = 3;
    paths.iter()
        .fold((-1, vec![]), |(min, mut paths), &path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let hs = node.borrow().data.game.state.playfield.grid.contour();
            let mut n = 0;
            for i in 0..hs.len() {
                let left = if i == 0 { true } else { (hs[i] as i8 - hs[i - 1] as i8).abs() >= TRENCH_HEIGHT };
                let right = if i == hs.len() - 1 { true } else { (hs[i] as i8 - hs[i + 1] as i8).abs() >= TRENCH_HEIGHT };
                if left && right {
                    n += 1;
                }
            }
            if min == -1 || n < min {
                (n, vec![path])
            } else if n == min {
                paths.push(path);
                (n, paths)
            } else {
                (min, paths)
            }
        }).1
}

fn _filter_by_contour<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    fn calc_stddev(values: &[i8]) -> f32 {
        let mean = values.iter().fold(0, |memo, v| memo + v) as f32 / values.len() as f32;
        let mut sum = 0.0;
        for v in values {
            sum += (*v as f32 - mean).powf(2.0);
        }
        (sum / values.len() as f32).sqrt()
    }

    paths.iter()
        .filter(|path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let hs = node.borrow().data.game.state.playfield.grid.contour();
            let stddev = calc_stddev(&hs);
            stddev < 5.0
        })
        .copied()
        .collect::<Vec<_>>()
}

fn _min_holds<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    paths.iter()
        .fold((-1, vec![]), |(min, mut paths), &path| {
            let n = deep_trinity_tree::ChildNodeIterator::new(root, path.iter())
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
        }).1
}

fn max_density<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    let path = paths.iter()
        .map(|&path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let density = node.borrow().data.game.state.playfield.grid.density_without_top_padding();
            ((density * 10000.0) as u32, path)
        })
        .max()
        .unwrap().1;
    vec![path]
}

fn _min_height<'a>(root: &Rc<RefCell<Node>>, paths: &[&'a deep_trinity_tree::Path]) -> Vec<&'a deep_trinity_tree::Path> {
    let path = paths.iter()
        .map(|&path| {
            let node = deep_trinity_tree::get(root, path.iter()).unwrap();
            let grid = &node.borrow().data.game.state.playfield.grid;
            (grid.height() - grid.top_padding(), path)
        })
        .min()
        .unwrap().1;
    vec![path]
}

//---

#[derive(Copy, Clone, Debug, Default)]
pub struct TreeBot {
    pub expansion_duration: std::time::Duration,
    pub num_expanded: usize,
}

impl Bot for TreeBot {
    fn think(&mut self, game: &Game<'static>) -> Result<Action, Box<dyn Error>> {
        let root = deep_trinity_tree::new(NodeData::new(None, game.clone(), false));
        let started_at = std::time::SystemTime::now();
        const NUM_EXPANSIONS: usize = 2;
        for _ in 0..NUM_EXPANSIONS {
            expand_leaves(&root)?;

            let initial_num = root.borrow().data.num_covered_empty_cells as i32;
            deep_trinity_tree::visit(&root, |node, _| {
                if !node.borrow().is_leaf() {
                    return deep_trinity_tree::VisitPlan::Children;
                }
                if node.borrow().data.num_covered_empty_cells as i32 - initial_num >= 3 {
                    node.borrow_mut().data.stop = true;
                }
                deep_trinity_tree::VisitPlan::Sibling
            });
        }
        // deep_trinity_tree::visit(&root, |node, _| {
        //     if !node.borrow().is_leaf() {
        //         return deep_trinity_tree::VisitPlan::Children;
        //     }
        //     println!("{}", node.borrow().data.game);
        //     deep_trinity_tree::VisitPlan::Sibling
        // });
        // assert!(false);

        // stats
        self.expansion_duration = std::time::SystemTime::now().duration_since(started_at)?;
        self.num_expanded = 0;
        deep_trinity_tree::visit(&root, |_, _| {
            self.num_expanded += 1;
            deep_trinity_tree::VisitPlan::Children
        });
        if self.num_expanded > 0 {
            self.num_expanded -= 1;
        }

        let paths = deep_trinity_tree::get_all_paths_to_leaves(&root);
        let paths = paths.iter().map(|path| path).collect::<Vec<_>>();

        let mut filter_chain = FilterChain::new(vec![
            to_box_filter(FunctionFilter(exclude_stopped)),
            // to_box_filter(OrFilter::new(to_box_filter_vec(&[tetris, hold_i, exclude_holds]))),
            to_box_filter(OrFilter::new(to_box_filter_vec(&[tetris, hold_i]))),
            to_box_filter(SuppressLineClear::new(10)),
            // to_box_filter(FunctionFilter(filter_by_contour)),
            // to_box_filter(FunctionFilter(min_trenches)),
            // to_box_filter(FunctionFilter(exclude_trenches)),
            to_box_filter(FunctionFilter(min_covered_empty_cells)),
            // to_box_filter(FunctionFilter(min_height)),
            to_box_filter(FunctionFilter(max_density)),
        ]);

        let paths = filter_chain.filter(&root, &paths);

        let path = paths.get(0).unwrap();
        let action = deep_trinity_tree::get(&root, [path.indices[0]].iter()).unwrap().borrow().data.by.unwrap();
        Ok(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BotRunner;

    #[test]
    fn test_tree_bot() {
        let seed = 0;
        let runner = BotRunner::new(5, true, Some(seed), false);
        let mut bot = TreeBot::default();
        let _game = runner.run_with_no_hooks(&mut bot).unwrap();
        // assert!(game.stats.lock > 40);
    }
}
