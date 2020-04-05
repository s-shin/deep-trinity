/// Move searcher by A* algorithm.
/// Using this, we can get mostly good moves to a specific placement.
use std::collections::{HashMap, BTreeMap, VecDeque};
use crate::{Move, FallingPiece, MovePathItem, Placement};
use super::{SearchConfiguration, MoveDestinations, SearchResult, MoveSearcher};

pub fn search_moves(conf: &SearchConfiguration, dst: Placement, debug: bool) -> SearchResult {
    type F = i16;
    type OpenList = BTreeMap<F, VecDeque<Placement>>;

    #[derive(Copy, Clone, Debug)]
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

    fn cost_func(start: &Placement, target: &Placement, mv: Move) -> F {
        const BUFFER: i8 = 0; // TODO
        match mv {
            Move::Shift(_) => 2 + (start.pos.1 + BUFFER - target.pos.1) as F,
            Move::Drop(_) => 1,
            Move::Rotate(_) => 3 + (start.pos.1 + BUFFER - target.pos.1) as F,
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
        debug_println!("target: placement: {:?}, f: {:?}, g: {}", target_placement, target_f, target_g);

        for mv in &MOVES {
            let mut fp = FallingPiece::new(conf.piece, target_placement);
            if fp.apply_move(*mv, conf.pf, conf.mode) {
                let f = target_g + cost_func(&conf.src, &target_placement, *mv) + heuristic_func(&fp.placement, &dst);
                if !open_list.contains_key(&f) {
                    open_list.insert(f, VecDeque::new());
                }
                let should_update = if let Some(ent) = state.get(&fp.placement) {
                    let r = f < ent.f;
                    debug_println!("  {:?} => placement: {:?}, f: {}, is_checked: {}, new_f: {} => update: {}",
                        mv, fp.placement, ent.f, ent.is_checked, f, r);
                    r
                } else {
                    debug_println!("  {:?} => placement: {:?}, new_f: {} => new", mv, fp.placement, f);
                    true
                };
                if should_update {
                    open_list.get_mut(&f).unwrap().push_back(fp.placement);
                    state.insert(fp.placement, StateEntry::new(f, false));
                    found.insert(fp.placement, MovePathItem::new(fp.move_path.items[0].by, fp.move_path.initial_placement));
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
    use crate::{Game, Piece, MovePlayer, upos, pos, ORIENTATION_3, ORIENTATION_1};
    use super::*;

    #[test]
    fn test1() {
        let mut game: Game = Default::default();
        let pf = &mut game.state.playfield;
        pf.set_rows(upos!(0, 0), &[
            "   @@@@   ",
            "@@@@@@    ",
            "@@@@@@@ @@",
        ]);
        game.supply_next_pieces(&[Piece::T]);
        game.setup_falling_piece(None).unwrap();
        let dst = Placement::new(ORIENTATION_3, pos!(6, 0));
        let r = game.search_moves(&mut AStarMoveSearcher::new(dst, false));
        assert!(r.is_ok());
        let r = r.unwrap();
        let path = r.get(&dst);
        assert!(path.is_some());
        let mut mp = MovePlayer::new(path.unwrap());
        while mp.step(&mut game).unwrap() {
            // println!("{}", game);
        }
        // println!("{}", game);
    }

    #[test]
    fn test2() {
        let mut game: Game = Default::default();
        let pf = &mut game.state.playfield;
        pf.set_rows(upos!(0, 0), &["@@@@@@@@@@"].repeat(20));
        game.supply_next_pieces(&[Piece::T]);
        game.setup_falling_piece(None).unwrap();
        let dst = Placement::new(ORIENTATION_1, pos!(3, 20));
        let r = game.search_moves(&mut AStarMoveSearcher::new(dst, false));
        assert!(r.is_ok());
        let r = r.unwrap();
        let path = r.get(&dst);
        assert!(path.is_some());
        let mut mp = MovePlayer::new(path.unwrap());
        while mp.step(&mut game).unwrap() {
            // println!("{}", game);
        }
        // println!("{}", game);
    }
}
