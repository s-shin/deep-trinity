/// Heuristic move searcher by brute force approach.
/// By using this, we can get at least all lockable placements.
use grid::Grid;
use crate::{Move, MovePathItem};
use super::{SearchConfiguration, SearchResult, MoveSearcher};

pub fn search_moves(conf: &SearchConfiguration, debug: bool) -> SearchResult {
    let highest = conf.pf.grid.height() - conf.pf.grid.top_padding();
    let piece_height = conf.piece_spec.grid(conf.src.orientation).height();
    let safe_y = conf.src.pos.1.min(highest + piece_height);

    let mut conf2 = conf.clone();
    conf2.src.pos.1 = safe_y;
    let mut r = super::bruteforce::search_moves(&conf2, debug);

    if safe_y != conf.src.pos.1 {
        r.found.insert(conf2.src, MovePathItem::new(Move::Drop(conf.src.pos.1 - safe_y), conf.src));
        r.src = conf.src;
    }

    r
}

#[derive(Copy, Clone, Debug, Default)]
pub struct HeuristicBruteForceMoveSearcher {
    debug: bool,
}

impl HeuristicBruteForceMoveSearcher {
    pub fn debug() -> Self { Self { debug: true } }
}

impl MoveSearcher for HeuristicBruteForceMoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult {
        search_moves(conf, self.debug)
    }
}

#[cfg(test)]
mod test {
    use crate::{Game, Piece, Placement, RotationMode, MovePlayer, Orientation1, Orientation3};
    use super::*;

    #[test]
    fn test_search_moves() {
        let mut game: Game = Game::default();
        game.supply_next_pieces(&[Piece::I]);
        game.setup_falling_piece(None).unwrap();
        let pf = &mut game.state.playfield;
        pf.set_rows_with_strs((0, 0).into(), &[
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
            " @@@@@@@@@",
        ]);
        let fp = game.state.falling_piece.as_ref().unwrap();
        let conf = SearchConfiguration::new(&pf, fp.piece_spec, fp.placement, RotationMode::Srs);
        let r = search_moves(&conf, false);
        let dst1 = Placement::new(Orientation1, (-2, 0).into());
        let path = r.get(&dst1);
        assert!(path.is_some());
        let dst2 = Placement::new(Orientation3, (-2, -1).into()); // alt
        assert!(r.get(&dst2).is_some());
        let print = false;
        if print { println!("{}", game); }
        let mut mp = MovePlayer::new(path.unwrap());
        while mp.step(&mut game).unwrap() {
            if print { println!("{}", game); }
        }
        if print { println!("{}", game); }
    }
}
