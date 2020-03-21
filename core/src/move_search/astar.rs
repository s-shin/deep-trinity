use crate::*;
use crate::move_search::*;

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

    // heuristic function.
    fn distance(a: &Placement, b: &Placement) -> F {
        let dp = a.pos - b.pos;
        (dp.0.abs() + dp.1.abs() + (b.orientation.id() as i8 - a.orientation.id() as i8).abs() * 2) as F
    }

    macro_rules! debug_println {
        ($e:expr $(, $es:expr)*) => {
            if debug {
                println!($e $(, $es)*);
            }
        }
    }

    struct Action {
        mv: Move,
        cost: F,
    }
    const ACTIONS: [Action; 5] = [
        Action { mv: Move::Drop(1), cost: 1 },
        Action { mv: Move::Shift(1), cost: 1 },
        Action { mv: Move::Shift(-1), cost: 1 },
        Action { mv: Move::Rotate(1), cost: 1 },
        Action { mv: Move::Rotate(-1), cost: 1 },
    ];

    let mut found: HashMap<Placement, MoveRecordItem> = HashMap::new();
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
        let target_g = target_f - distance(&target_placement, &dst);
        debug_println!("target: placement: {:?}, f: {:?}, (g: {})", target_placement, target_f, target_g);

        for action in &ACTIONS {
            let mut fp = FallingPiece::new(conf.piece, target_placement);
            if fp.apply_move(action.mv, conf.pf, conf.mode) {
                let f = target_g + action.cost + distance(&fp.placement, &dst);
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
                    action.mv, fp.placement, next.f, next.is_checked, f, should_update);
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
