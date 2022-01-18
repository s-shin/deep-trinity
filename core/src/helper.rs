use std::collections::HashSet;
use std::rc::Rc;
use crate::{Game, MoveTransition, FallingPiece, Playfield, GameRules, Piece, MovePathItem, Move, MovePath, LineClear, RotationMode, Placement, ORIENTATION_1, ORIENTATION_2, ORIENTATION_3, ORIENTATION_0};
use crate::move_search::{MoveSearcher, SearchConfiguration, SearchResult};
use crate::move_search::bruteforce::BruteForceMoveSearcher;
use crate::move_search::humanly_optimized::HumanlyOptimizedMoveSearcher;
use crate::move_search::astar::AStarMoveSearcher;

pub fn get_alternative_placements(piece: Piece, placement: &Placement) -> Vec<Placement> {
    match piece {
        Piece::O => {
            match placement.orientation {
                ORIENTATION_0 => vec![
                    Placement::new(ORIENTATION_1, placement.pos + (0, 1).into()),
                    Placement::new(ORIENTATION_2, placement.pos + (1, 1).into()),
                    Placement::new(ORIENTATION_3, placement.pos + (1, 0).into()),
                ],
                ORIENTATION_1 => vec![
                    Placement::new(ORIENTATION_0, placement.pos + (0, -1).into()),
                    Placement::new(ORIENTATION_2, placement.pos + (1, 0).into()),
                    Placement::new(ORIENTATION_3, placement.pos + (1, -1).into()),
                ],
                ORIENTATION_2 => vec![
                    Placement::new(ORIENTATION_0, placement.pos + (-1, -1).into()),
                    Placement::new(ORIENTATION_1, placement.pos + (-1, 0).into()),
                    Placement::new(ORIENTATION_3, placement.pos + (0, -1).into()),
                ],
                ORIENTATION_3 => vec![
                    Placement::new(ORIENTATION_0, placement.pos + (-1, 0).into()),
                    Placement::new(ORIENTATION_1, placement.pos + (-1, 1).into()),
                    Placement::new(ORIENTATION_2, placement.pos + (0, 1).into()),
                ],
                _ => panic!(),
            }
        }
        Piece::I => {
            match placement.orientation {
                ORIENTATION_0 => vec![
                    Placement::new(ORIENTATION_2, placement.pos + (1, 0).into()),
                ],
                ORIENTATION_1 => vec![
                    Placement::new(ORIENTATION_3, placement.pos + (0, -1).into()),
                ],
                ORIENTATION_2 => vec![
                    Placement::new(ORIENTATION_0, placement.pos + (-1, 0).into()),
                ],
                ORIENTATION_3 => vec![
                    Placement::new(ORIENTATION_1, placement.pos + (0, 1).into()),
                ],
                _ => panic!(),
            }
        }
        Piece::S | Piece::Z => {
            match placement.orientation {
                ORIENTATION_0 => vec![
                    Placement::new(ORIENTATION_2, placement.pos + (0, 1).into()),
                ],
                ORIENTATION_1 => vec![
                    Placement::new(ORIENTATION_3, placement.pos + (1, 0).into()),
                ],
                ORIENTATION_2 => vec![
                    Placement::new(ORIENTATION_0, placement.pos + (0, -1).into()),
                ],
                ORIENTATION_3 => vec![
                    Placement::new(ORIENTATION_1, placement.pos + (-1, 0).into()),
                ],
                _ => panic!(),
            }
        }
        _ => vec![],
    }
}

pub fn get_nearest_alternative_placement(piece: Piece, target: &Placement, src: &Placement,
                                         distance_factors: Option<(usize, usize, usize)>) -> Placement {
    let mut candidate = target.clone();
    let mut distance = src.distance(target, distance_factors);
    for p in &get_alternative_placements(piece, target) {
        let d = src.distance(p, distance_factors);
        if d < distance {
            distance = d;
            candidate = p.clone();
        }
    }
    candidate
}

//---

pub struct MoveDecisionStuff {
    /// Movable and lockable placements including all alternative placements.
    pub dst_candidates: HashSet<Placement>,
    /// The result of the search by [BruteForceMoveSearcher].
    pub brute_force_search_result: SearchResult,
}

impl MoveDecisionStuff {
    pub fn new<'a>(pf: &Playfield<'a>, fp: &FallingPiece<'a>, rules: &GameRules) -> Self {
        let mut searcher: BruteForceMoveSearcher = Default::default();
        let conf = SearchConfiguration::new(pf, fp.piece_spec, fp.placement, rules.rotation_mode);
        let search_result = searcher.search(&conf);
        let dst_candidates = pf.search_lockable_placements(fp.piece_spec).iter()
            .filter(|&p| search_result.contains(p))
            .copied()
            .collect::<HashSet<_>>();
        Self {
            dst_candidates,
            brute_force_search_result: search_result,
        }
    }
}

pub struct MoveDecisionHelper<'a> {
    pub falling_piece: &'a FallingPiece<'a>,
    pub playfield: &'a Playfield<'a>,
    pub rules: &'a GameRules,
    pub stuff: Rc<MoveDecisionStuff>,
}

impl<'a> MoveDecisionHelper<'a> {
    pub fn new(pf: &'a Playfield<'a>, fp: &'a FallingPiece<'a>, rules: &'a GameRules, stuff: Option<Rc<MoveDecisionStuff>>) -> Self {
        Self {
            playfield: pf,
            falling_piece: fp,
            rules,
            stuff: stuff.unwrap_or_else(|| Rc::new(MoveDecisionStuff::new(pf, fp, rules))),
        }
    }
    pub fn with_game(game: &'a Game<'a>, stuff: Option<Rc<MoveDecisionStuff>>) -> Result<Self, &'static str> {
        if matches!(game.state.falling_piece, None) {
            return Err("The falling_piece should not be None.");
        }
        Ok(Self::new(&game.state.playfield, game.state.falling_piece.as_ref().unwrap(), &game.rules, stuff))
    }
    pub fn tspin_moves(&self) -> Result<Vec<(MoveTransition, LineClear)>, &'static str> {
        if self.falling_piece.piece() != Piece::T {
            return Err("This helper is not for T piece.");
        }
        let mut r = vec![];
        for dst in self.stuff.dst_candidates.iter() {
            let fp = FallingPiece::new(self.falling_piece.piece_spec, *dst);
            for cw in &[true, false] {
                for src in self.playfield.check_reverse_rotation(self.rules.rotation_mode, &fp, *cw).iter() {
                    if !self.stuff.brute_force_search_result.contains(src) {
                        continue;
                    }
                    let mt = MoveTransition::new(*dst, Some(MovePathItem::new(Move::Rotate(if *cw { 1 } else { -1 }), *src)));
                    let line_clear = self.playfield.check_line_clear(
                        &FallingPiece::new_with_last_move_transition(self.falling_piece.piece_spec, &mt),
                        self.rules.tspin_judgement_mode);
                    if line_clear.tspin.is_none() {
                        continue;
                    }
                    r.push((mt, line_clear));
                }
            }
        }
        Ok(r)
    }
    pub fn tetris_destinations(&self) -> Result<Vec<Placement>, &'static str> {
        if self.falling_piece.piece() != Piece::I {
            return Err("This helper is not for I piece.");
        }
        let r = self.stuff.dst_candidates.iter()
            .filter(|&p| {
                if p.orientation.is_even() {
                    return false;
                }
                let fp = FallingPiece::new(self.falling_piece.piece_spec, *p);
                let line_clear = self.playfield.check_line_clear(&fp, self.rules.tspin_judgement_mode);
                line_clear.is_tetris()
            })
            .copied()
            .collect::<Vec<_>>();
        Ok(r)
    }
}

//---

#[deprecated]
pub fn get_move_candidates(pf: &Playfield, fp: &FallingPiece, rules: &GameRules) -> HashSet<MoveTransition> {
    let lockable = pf.search_lockable_placements(fp.piece_spec);
    let mut searcher: BruteForceMoveSearcher = Default::default();
    let conf = SearchConfiguration::new(pf, fp.piece_spec, fp.placement, rules.rotation_mode);
    let search_result = searcher.search(&conf);

    let mut r = HashSet::new();
    for p in lockable.iter() {
        if search_result.contains(p) {
            if fp.piece_spec.piece == Piece::T {
                let mut pp = p.clone();
                pp.pos.1 += 1;
                if search_result.contains(&pp) {
                    r.insert(MoveTransition::new(*p, Some(MovePathItem::new(Move::Drop(1), pp))));
                }
                // Append worthy transitions by rotation.
                let dst_fp = FallingPiece::new(fp.piece_spec, *p);
                for cw in &[true, false] {
                    for src in pf.check_reverse_rotation(rules.rotation_mode, &dst_fp, *cw).iter() {
                        if let Some(_) = pf.check_tspin(
                            &FallingPiece::new_with_one_path_item(
                                fp.piece_spec, *src, Move::Rotate(if *cw { 1 } else { -1 }), *p),
                            rules.tspin_judgement_mode,
                        ) {
                            r.insert(MoveTransition::new(
                                *p,
                                Some(MovePathItem::new(
                                    Move::Rotate(if *cw { 1 } else { -1 }),
                                    *src,
                                )),
                            ));
                        }
                    }
                }
            } else {
                r.insert(MoveTransition::new(*p, None));
            }
        }
    }
    r
}

pub fn get_almost_good_move_path(rotation_mode: RotationMode, pf: &Playfield, fp: &FallingPiece, dst: &Placement) -> Option<MovePath> {
    let search_conf = SearchConfiguration::new(pf, fp.piece_spec, fp.placement, rotation_mode);

    // Since HumanlyOptimizedMoveSearcher is better performance than other searchers,
    // search moves by it first.
    let r = HumanlyOptimizedMoveSearcher::new(*dst, true).search(&search_conf);
    if let Some(path) = r.get(dst) {
        return Some(path);
    }

    // Search moves by A* searcher.
    let r = AStarMoveSearcher::new(*dst, false).search(&search_conf);
    let path_by_aster = if let Some(path) = r.get(dst) {
        path
    } else {
        // Must be found if reachable placement given.
        return None;
    };

    // Detect the last position where can be reached by only drop moves from around the spawned position.
    if let Some((i, item)) = path_by_aster.items.iter().enumerate().rev().find(|(_, item)| {
        let n = fp.piece_spec.initial_placement.pos.1 - item.placement.pos.1;
        if n < 0 {
            return false;
        }
        pf.can_raise_n(fp, n)
    }) {
        let r = HumanlyOptimizedMoveSearcher::new(item.placement.clone(), true).search(&search_conf);
        if let Some(mut path) = r.get(&item.placement) {
            if i <= path_by_aster.len() - 2 {
                for j in (i + 1)..path_by_aster.len() {
                    path.merge_or_push(path_by_aster.items[j]);
                }
            }
            return Some(path);
        }
    }

    Some(path_by_aster)
}

#[deprecated]
pub fn get_almost_good_move_path_old(pf: &Playfield, fp: &FallingPiece, last_transition: &MoveTransition, rotation_mode: RotationMode) -> Option<MovePath> {
    enum Searcher {
        HumanOptimized,
        AStar,
    }

    let mut patterns = Vec::new();
    {
        let dst = if let Some(hint) = last_transition.hint { hint.placement } else { last_transition.placement };
        // Try to find the path with fewest rotations.
        let alt = get_nearest_alternative_placement(fp.piece(), &dst, &fp.placement, Some((2, 1, 1)));
        patterns.push((alt, Searcher::HumanOptimized));
        patterns.push((alt, Searcher::AStar));
        if dst != alt {
            // In a move with special rotations, a piece cannot always be reached to an alternative placement,
            // so also check the original destination.
            patterns.push((dst, Searcher::AStar));
        }
    }

    let mut path = None;
    let search_conf = SearchConfiguration::new(pf, fp.piece_spec, fp.placement, rotation_mode);
    for (dst, searcher) in patterns.iter() {
        let r = match *searcher {
            Searcher::HumanOptimized => {
                let mut searcher = HumanlyOptimizedMoveSearcher::new(*dst, true);
                searcher.search(&search_conf)
            }
            Searcher::AStar => {
                let mut searcher = AStarMoveSearcher::new(*dst, false);
                searcher.search(&search_conf)
            }
        };
        if let Some(mut p) = r.get(dst) {
            if let Some(hint) = last_transition.hint {
                p.merge_or_push(MovePathItem::new(hint.by, last_transition.placement));
            }
            path = Some(p);
            break;
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_decision_helper() {
        let mut pf: Playfield<'static> = Default::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            "  @@  @@@@",
            "@@@   @@@@",
            "@@@@ @@@@@",
            "@@@@ @@@@@",
            "@@@@ @@@@@",
            "@@@@ @@@@@",
            "@@@@ @@@@@",
        ]);
        let rules: GameRules = Default::default();
        {
            let fp = FallingPiece::spawn(Piece::T.default_spec(), Some(&pf));
            let h = MoveDecisionHelper::new(&pf, &fp, &rules);
            let moves = h.tspin_moves().unwrap();
            assert_eq!(10, moves.len());
        }
        {
            let fp = FallingPiece::spawn(Piece::I.default_spec(), Some(&pf));
            let h = MoveDecisionHelper::new(&pf, &fp, &rules);
            let dsts = h.tetris_destinations().unwrap();
            assert_eq!(2, dsts.len());
        }
    }

    #[test]
    fn test_get_almost_good_move_path() {
        let mut pf: Playfield<'static> = Default::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            "@@  @     ",
            "@   @     ",
            "@ @@@     ",
            "@  @@     ",
            "@   @     ",
            "@@ @@@    ",
        ]);
        let fp = FallingPiece::spawn(Piece::T.default_spec(), Some(&pf));
        let dst = Placement::new(ORIENTATION_2, (1, 0).into());
        let path = get_almost_good_move_path(RotationMode::Srs, &pf, &fp, &dst).unwrap();
        // for i in 0..path.len() {
        //     println!("{:?}", path.items[i]);
        // }
        // MovePathItem { by: Shift(-1), placement: Placement { orientation: Orientation(0), pos: Vec2(2, 18) } }
        // MovePathItem { by: Rotate(-1), placement: Placement { orientation: Orientation(3), pos: Vec2(2, 18) } }
        // MovePathItem { by: Drop(14), placement: Placement { orientation: Orientation(3), pos: Vec2(2, 4) } }
        // MovePathItem { by: Rotate(1), placement: Placement { orientation: Orientation(0), pos: Vec2(1, 3) } }
        // MovePathItem { by: Rotate(1), placement: Placement { orientation: Orientation(1), pos: Vec2(0, 1) } }
        // MovePathItem { by: Rotate(1), placement: Placement { orientation: Orientation(2), pos: Vec2(1, 0) } }
        assert_eq!(6, path.len());
    }
}
