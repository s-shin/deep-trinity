use std::collections::{HashMap, BTreeMap, VecDeque};
use crate::{Move, FallingPiece, MoveRecordItem, Placement};
use super::{SearchConfiguration, MoveDestinations, SearchResult, MoveSearcher};

pub fn search_moves(conf: &SearchConfiguration, dst: Placement, debug: bool) -> SearchResult {
    type F = i16;
    type OpenList = BTreeMap<F, VecDeque<Placement>>;

    #[derive(Copy, Clone)]
    struct StateEntry {
        f: F,
        is_checked: bool,
    }
    impl StateEntry {
        fn new(f: F, is_checked: bool) -> Self { Self { f, is_checked } }
    }

    fn heuristic_func(current: &Placement, target: &Placement) -> F {
        current.distance(target, Some((1, 1, 1))) as F
    }

    fn cost_func(mv: Move, prev: Option<Move>) -> F {
        match mv {
            Move::Rotate(_) => match prev {
                // To prevent cases such as `drop -> rotate -> shift -> drop` instead of `rotate -> shift -> drop`,
                // costs are adjusted.
                Some(Move::Drop(_)) => 4,
                Some(Move::Shift(_)) => 3,
                _ => 2,
            },
            Move::Drop(_) => match prev {
                Some(Move::Rotate(_)) => 2,
                _ => 1,
            },
            Move::Shift(_) => match prev {
                Some(Move::Drop(_)) => 2,
                _ => 1,
            }
        }
    }

    macro_rules! debug_println {
        ($e:expr $(, $es:expr)*) => {
            if debug {
                println!($e $(, $es)*);
            }
        }
    }

    const MOVES: [Move; 5] = [
        Move::Drop(1), Move::Shift(1), Move::Shift(-1), Move::Rotate(1), Move::Rotate(-1),
    ];

    let mut found = MoveDestinations::new();
    let mut open_list = OpenList::new();
    let mut state: HashMap<Placement, StateEntry> = HashMap::new();

    open_list.insert(0, VecDeque::from(vec![conf.src]));
    state.insert(conf.src, StateEntry::new(0, false));

    loop {
        let mut target: Option<(F, Placement)> = None;
        for (f, placements) in open_list.iter_mut() {
            while let Some(p) = placements.pop_front() {
                if let Some(ent) = state.get_mut(&p) {
                    if ent.is_checked {
                        continue;
                    }
                    ent.is_checked = true;
                }
                target = Some((*f, p));
                break;
            }
            if target.is_some() {
                break;
            }
        }
        if target.is_none() {
            debug_println!("target not found.");
            break;
        }
        let (target_f, target_placement) = target.unwrap();
        if target_placement == dst {
            debug_println!("target found.");
            break;
        }
        let target_g = target_f - heuristic_func(&target_placement, &dst);
        let target_by = found.get(&target_placement).map(|item| { item.by });
        debug_println!("target: placement: {:?}, f: {:?}, (g: {})", target_placement, target_f, target_g);

        for mv in &MOVES {
            // Opposite moves should be restricted since it leads circular result.
            match mv {
                Move::Shift(n1) => match target_by {
                    Some(Move::Shift(n2)) => if n1 * n2 < 0 { continue; },
                    _ => {}
                }
                Move::Rotate(n1) => match target_by {
                    Some(Move::Rotate(n2)) => if n1 * n2 < 0 { continue; },
                    _ => {}
                }
                _ => {}
            }
            let mut fp = FallingPiece::new(conf.piece, target_placement);
            if fp.apply_move(*mv, conf.pf, conf.mode) {
                let f = target_g + cost_func(*mv, target_by) + heuristic_func(&fp.placement, &dst);
                if !open_list.contains_key(&f) {
                    open_list.insert(f, VecDeque::new());
                }
                let next = if let Some(ent) = state.get(&fp.placement) {
                    *ent
                } else {
                    state.insert(fp.placement, StateEntry::new(0, false));
                    StateEntry::new(0, false)
                };
                let should_update = !next.is_checked || f < next.f;
                debug_println!("  {:?} => placement: {:?}, f: {}, is_checked: {}, new_f: {} => update: {}",
                    mv, fp.placement, next.f, next.is_checked, f, should_update);
                if should_update {
                    open_list.get_mut(&f).unwrap().push_back(fp.placement);
                    state.insert(fp.placement, StateEntry::new(f, false));
                    found.insert(fp.placement, MoveRecordItem::new(fp.move_record.items[0].by, fp.move_record.initial_placement));
                }
            }
        }
    }

    SearchResult { src: conf.src, found }
}

#[derive(Copy, Clone, Debug)]
pub struct AStarMoveSearcher {
    dst: Placement,
    debug: bool,
}

impl AStarMoveSearcher {
    pub fn new(dst: Placement, debug: bool) -> Self {
        Self { dst, debug }
    }
}

impl MoveSearcher for AStarMoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult {
        search_moves(conf, self.dst, self.debug)
    }
}

#[cfg(test)]
mod test {
    use crate::{Game, Piece, RotationMode, MovePlayer, upos, pos, ORIENTATION_3};
    use super::*;

    #[test]
    fn test_search_moves() {
        let mut game: Game = Game::default();
        game.supply_next_pieces(&[Piece::T]);
        game.setup_falling_piece(None).unwrap();
        let pf = &mut game.state.playfield;
        pf.set_rows(upos!(0, 0), &[
            "   ####   ",
            "######    ",
            "####### ##",
        ]);
        let fp = game.state.falling_piece.as_ref().unwrap();
        let conf = SearchConfiguration::new(&pf, fp.piece, fp.placement, RotationMode::Srs);
        let dst = Placement::new(ORIENTATION_3, pos!(6, 0));
        let r = search_moves(&conf, dst, false);
        let rec = r.get(&dst);
        assert!(rec.is_some());
        let mut mp = MovePlayer::new(rec.unwrap());
        while mp.step(&mut game).unwrap() {
            // println!("{}", game);
        }
        // println!("{}", game);
    }
}
