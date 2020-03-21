use crate::{Move, FallingPiece, MoveRecordItem, Placement};
use super::{SearchConfiguration, MoveDestinations, SearchResult, MoveSearcher};

pub const END: i8 = 100;

pub fn das_optim_plan() -> Vec<Vec<Move>> {
    vec![
        vec![Move::Shift(END), Move::Shift(-END)],
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
        vec![Move::Shift(0), Move::Shift(1), Move::Shift(-1)],
        vec![Move::Drop(END)],
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
    ]
}

pub fn edge_plan() -> Vec<Vec<Move>> {
    vec![
        vec![Move::Rotate(1), Move::Rotate(-1), Move::Rotate(2)],
        vec![Move::Shift(END), Move::Shift(-END)],
        vec![Move::Drop(END)],
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
    ]
}

pub fn normal_plan(conf: &SearchConfiguration) -> Vec<Vec<Move>> {
    let pf = &conf.pf;
    let fp = FallingPiece::new(conf.piece, conf.src);
    let num_r = pf.num_shiftable_cols(&fp, true) as i8;
    let num_l = pf.num_shiftable_cols(&fp, false) as i8;
    let mut first_mvs = Vec::new();
    for x in 1..=num_r {
        first_mvs.push(Move::Shift(x))
    }
    for x in 1..=num_l {
        first_mvs.push(Move::Shift(-x));
    }
    vec![
        first_mvs,
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
        vec![Move::Drop(END)],
        vec![Move::Rotate(0), Move::Rotate(1), Move::Rotate(-1)],
    ]
}

fn enumerate_index_patterns(patterns: &[usize], a: Vec<usize>, out: &mut Vec<Vec<usize>>) {
    if a.len() >= patterns.len() {
        out.push(a);
        return;
    }
    for i in 0..patterns[a.len()] {
        let mut a = a.clone();
        a.push(i);
        enumerate_index_patterns(patterns, a, out);
    }
}

fn search_moves(conf: &SearchConfiguration, plan: &[Vec<Move>]) -> SearchResult {
    let mut found = MoveDestinations::new();
    let pf = &conf.pf;
    let fp = FallingPiece::new(conf.piece, conf.src);

    let mut idx_patterns = Vec::new();
    enumerate_index_patterns(
        &plan.iter().map(|v| { v.len() }).collect::<Vec<usize>>(),
        vec![],
        &mut idx_patterns,
    );

    for indices in &idx_patterns {
        let mut fp = fp.clone();
        for (i, j) in indices.iter().enumerate() {
            let mut mv = plan[i][*j];
            mv = match mv {
                Move::Drop(100) => Move::Drop(pf.num_droppable_rows(&fp) as i8),
                Move::Shift(100) => Move::Shift(pf.num_shiftable_cols(&fp, true) as i8),
                Move::Shift(-100) => Move::Shift(pf.num_shiftable_cols(&fp, false) as i8),
                _ => mv,
            };
            let ok = fp.apply_move(mv, pf, conf.mode);
            if !ok {
                break;
            }
            let should_update = !found.contains_key(&fp.placement);
            if should_update {
                let item = if let Some((p, _)) = fp.last_two_placements() {
                    MoveRecordItem::new(mv, p)
                } else {
                    MoveRecordItem::new(mv, fp.move_record.initial_placement)
                };
                found.insert(fp.placement, item);
            }
        }
    }

    SearchResult::new(conf.src, found)
}

#[derive(Copy, Clone, Debug)]
pub struct HumanlyOptimizedMoveSearcher {
    dst: Placement,
    prefer_drop: bool,
    das_optim: bool,
}

impl HumanlyOptimizedMoveSearcher {
    pub fn new(dst: Placement, prefer_drop: bool, das_optim: bool) -> Self {
        Self { dst, prefer_drop, das_optim }
    }
}

impl MoveSearcher for HumanlyOptimizedMoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult {
        let mut candidate = None;
        for i in 0..2 {
            let r = match i {
                0 => {
                    if self.das_optim {
                        continue;
                    }
                    search_moves(conf, &das_optim_plan())
                }
                1 => {
                    search_moves(conf, &edge_plan())
                }
                // 2 => {
                //     search_moves(conf, &normal_plan(conf))
                // }
                _ => panic!(),
            };
            if let Some(rec) = r.get(&self.dst) {
                if !self.prefer_drop {
                    return r;
                }
                if let Some(Move::Drop(_)) = rec.items.last().map(|item| { item.by }) {
                    return r;
                } else {
                    candidate = Some(r);
                }
            }
        }
        candidate.unwrap_or(SearchResult::new(conf.src, MoveDestinations::new()))
    }
}

