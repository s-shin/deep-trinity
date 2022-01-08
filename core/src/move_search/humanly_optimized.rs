use crate::{Move, FallingPiece, MovePathItem, Placement};
use super::{SearchConfiguration, MoveDestinations, SearchResult, MoveSearcher};

pub const END: i8 = 100;

pub fn das_optim_plan() -> Vec<Vec<Move>> {
    vec![
        vec![Move::Shift(END), Move::Shift(-END)],
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
        // vec![Move::Shift(0), Move::Shift(1), Move::Shift(-1)],
        vec![Move::Drop(END)],
    ]
}

pub fn edge_plan() -> Vec<Vec<Move>> {
    vec![
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1), Move::Rotate(2), Move::Rotate(2)],
        vec![Move::Shift(0), Move::Shift(END), Move::Shift(-END)],
        vec![Move::Drop(END)],
    ]
}

pub fn normal_plan(conf: &SearchConfiguration) -> Vec<Vec<Move>> {
    let pf = &conf.pf;
    let fp = FallingPiece::new(conf.piece_spec, conf.src);
    let num_r = pf.num_shiftable_cols(&fp, true) as i8;
    let num_l = pf.num_shiftable_cols(&fp, false) as i8;
    // Move::Shift(0) is checked in edge_plan.
    let mut first_moves = Vec::with_capacity((num_r + num_l) as usize);
    for x in 1..=num_r {
        first_moves.push(Move::Shift(x))
    }
    for x in 1..=num_l {
        first_moves.push(Move::Shift(-x));
    }
    vec![
        first_moves,
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
        vec![Move::Drop(END)],
    ]
}

fn get_all_index_patterns<T>(vs: &[Vec<T>]) -> Vec<Vec<usize>> {
    if vs.is_empty() {
        return Vec::new();
    }
    let lens = vs.iter().map(|v| v.len()).collect::<Vec<_>>();
    let num_patterns = lens.iter().copied().reduce(|accum, len| accum * len).unwrap();
    let mut patterns = Vec::with_capacity(num_patterns);
    for n in 0..num_patterns {
        let indices = lens.iter().copied().map(|len| n % len).collect::<Vec<_>>();
        patterns.push(indices);
    }
    patterns
}

fn search_moves(conf: &SearchConfiguration, plan: &[Vec<Move>]) -> SearchResult {
    let mut found = MoveDestinations::new();
    let pf = &conf.pf;
    let fp = FallingPiece::new(conf.piece_spec, conf.src);

    let index_patterns = get_all_index_patterns(plan);

    for indices in &index_patterns {
        let mut fp = fp.clone();
        for (i, j) in indices.iter().enumerate() {
            let mut mv = plan[i][*j];
            mv = match mv {
                Move::Drop(100) => Move::Drop(pf.num_droppable_rows(&fp) as i8),
                Move::Shift(100) => Move::Shift(pf.num_shiftable_cols(&fp, true) as i8),
                Move::Shift(-100) => Move::Shift(-(pf.num_shiftable_cols(&fp, false) as i8)),
                _ => mv,
            };
            let ok = fp.apply_move(mv, pf, conf.mode);
            if !ok {
                break;
            }
            let should_update = !found.contains_key(&fp.placement);
            if should_update {
                let item = if let Some(mt) = fp.last_move_transition(true) {
                    MovePathItem::new(mv, mt.hint.unwrap().placement)
                } else {
                    MovePathItem::new(mv, fp.move_path.initial_placement)
                };
                found.insert(fp.placement, item);
            }
        }
    }

    SearchResult::new(conf.src, found)
}

/// This searcher covers most of moves the last of which is hard drop.
#[derive(Copy, Clone, Debug)]
pub struct HumanlyOptimizedMoveSearcher {
    dst: Placement,
    das_optim: bool,
}

impl HumanlyOptimizedMoveSearcher {
    pub fn new(dst: Placement, das_optim: bool) -> Self {
        Self { dst, das_optim }
    }
}

impl MoveSearcher for HumanlyOptimizedMoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult {
        for i in 0..=2 {
            let r = match i {
                0 => {
                    if !self.das_optim {
                        continue;
                    }
                    search_moves(conf, &das_optim_plan())
                }
                1 => {
                    search_moves(conf, &edge_plan())
                }
                2 => {
                    search_moves(conf, &normal_plan(conf))
                }
                _ => panic!(),
            };
            if let Some(_) = r.get(&self.dst) {
                return r;
            }
        }
        SearchResult::new(conf.src, MoveDestinations::new())
    }
}

#[cfg(test)]
mod test {
    use crate::*;
    use super::*;

    #[test]
    fn test_get_all_index_patterns() {
        let index_patterns = get_all_index_patterns(&[
            vec![1, 2, 3],
            vec![4, 5],
            vec![6],
        ]);
        assert_eq!(vec![
            vec![0, 0, 0],
            vec![1, 1, 0],
            vec![2, 0, 0],
            vec![0, 1, 0],
            vec![1, 0, 0],
            vec![2, 1, 0],
        ], index_patterns);
    }

    #[test]
    fn test_das_optim_plan() {
        let plan = das_optim_plan();
        let pf = Playfield::default();
        let fp = FallingPiece::spawn(Piece::I.default_spec(), None);
        let conf = SearchConfiguration::new(&pf, fp.piece_spec, fp.placement, RotationMode::Srs);
        let r = search_moves(&conf, &plan);
        for p in r.found.keys() {
            // println!("{:?}", p);
            let mut g = Game::default();
            g.state.playfield.grid.put(p.pos, FallingPiece::new(fp.piece_spec, *p).grid());
            // println!("{}", g);
        }
    }
}
