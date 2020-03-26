use crate::{Move, FallingPiece, MoveRecordItem};
use super::{SearchConfiguration, MoveDestinations, SearchResult, MoveSearcher};

const MOVES: [Move; 5] = [Move::Drop(1), Move::Shift(1), Move::Shift(-1), Move::Rotate(1), Move::Rotate(-1)];

pub fn search_moves(conf: &SearchConfiguration, debug: bool) -> SearchResult {
    let mut found = MoveDestinations::new();

    fn search(conf: &SearchConfiguration, fp: &FallingPiece, depth: usize, found: &mut MoveDestinations, debug: bool) {
        macro_rules! debug_println {
            ($e:expr $(, $es:expr)*) => {
                if debug {
                    if depth > 0 {
                        print!("{}", "│".repeat(depth));
                    }
                    println!($e $(, $es)*);
                }
            }
        }

        debug_println!("search_all: {:?} {}", fp.placement.orientation, fp.placement.pos);
        if depth > 0 && fp.placement == conf.src {
            debug_println!("=> initial placement.");
            return;
        }
        if found.contains_key(&fp.placement) {
            debug_println!("=> already checked.");
            return;
        }
        debug_assert!(fp.move_record.len() <= 1);
        if let Some(last) = fp.move_record.last() {
            let from = MoveRecordItem::new(last.by, fp.move_record.initial_placement);
            let v = found.insert(fp.placement, from);
            debug_assert!(v.is_none());
        }

        let mut fp = FallingPiece::new(fp.piece, fp.placement);
        for mv in &MOVES {
            debug_println!("├ {:?}", mv);
            if fp.apply_move(*mv, conf.pf, conf.mode) {
                search(conf, &fp, depth + 1, found, debug);
                fp.rollback();
            }
        }
        debug_println!("=> checked.");
    };

    search(conf, &FallingPiece::new(conf.piece, conf.src), 0, &mut found, debug);

    SearchResult { src: conf.src, found }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct BruteForceMoveSearcher {
    debug: bool
}

impl BruteForceMoveSearcher {
    pub fn debug() -> Self { Self { debug: true } }
}

impl MoveSearcher for BruteForceMoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult {
        search_moves(conf, self.debug)
    }
}

