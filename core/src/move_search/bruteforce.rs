/// Move searcher by brute force approach.
/// By using this, we can get all movable placements.
///
/// Remarks: Since this searcher doesn't search all move transitions,
/// the result will lack some meaningful special rotations (e.g. T-Spin Mini).
use crate::{Move, FallingPiece, MovePathItem};
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
        debug_assert!(fp.move_path.len() <= 1);
        if let Some(last) = fp.move_path.last() {
            let from = MovePathItem::new(last.by, fp.move_path.initial_placement);
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
    }
    ;

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

#[cfg(test)]
mod test {
    use crate::{Game, Piece, Placement, RotationMode, MovePlayer, upos, pos, ORIENTATION_1};
    use super::*;

    #[test]
    fn test_search_moves() {
        let mut game: Game = Game::default();
        game.supply_next_pieces(&[Piece::I]);
        game.setup_falling_piece(None).unwrap();
        let pf = &mut game.state.playfield;
        pf.set_rows(upos!(0, 0), &[
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
        ]);
        let fp = game.state.falling_piece.as_ref().unwrap();
        let conf = SearchConfiguration::new(&pf, fp.piece, fp.placement, RotationMode::Srs);
        let dst = Placement::new(ORIENTATION_1, pos!(-2, 0));
        let r = search_moves(&conf, false);
        let path = r.get(&dst);
        assert!(path.is_some());
        let mut mp = MovePlayer::new(path.unwrap());
        while mp.step(&mut game).unwrap() {
            println!("{}", game);
        }
        println!("{}", game);
    }
}
