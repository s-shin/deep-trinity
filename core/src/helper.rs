use std::collections::HashSet;
use crate::{MoveTransition, FallingPiece, Playfield, move_search, GameRules, Piece, MovePathItem, Move, MovePath, LineClear, RotationMode, PieceSpec, Placement, ORIENTATION_1, ORIENTATION_2, ORIENTATION_3, ORIENTATION_0};
use crate::move_search::{MoveSearcher, SearchConfiguration};

pub fn get_placement_aliases(piece: Piece, placement: &Placement) -> Vec<Placement> {
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

pub fn get_nearest_placement_alias(piece: Piece, aliased: &Placement, reference: &Placement,
                                   factors: Option<(usize, usize, usize)>) -> Placement {
    let mut candidate = aliased.clone();
    let mut distance = reference.distance(aliased, factors);
    for p in &get_placement_aliases(piece, aliased) {
        let d = reference.distance(p, factors);
        if d < distance {
            distance = d;
            candidate = p.clone();
        }
    }
    candidate
}

pub fn get_move_candidates(pf: &Playfield, fp: &FallingPiece, rules: &GameRules) -> HashSet<MoveTransition> {
    use move_search::bruteforce::BruteForceMoveSearcher;

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

pub fn get_almost_good_move_path(pf: &Playfield, fp: &FallingPiece, last_transition: &MoveTransition, rotation_mode: RotationMode) -> Option<MovePath> {
    use move_search::humanly_optimized::HumanlyOptimizedMoveSearcher;
    use move_search::astar::AStarMoveSearcher;

    enum Searcher {
        HumanOptimized,
        AStar,
    }

    // NOTE: For special rotations, we should also check the original destination.
    let mut patterns = Vec::new();
    if let Some(hint) = last_transition.hint {
        let dst1 = hint.placement;
        let dst2 = get_nearest_placement_alias(fp.piece(), &dst1, &fp.placement, None);
        if let Move::Rotate(_) = hint.by {
            patterns.push((dst2, Searcher::HumanOptimized));
        }
        patterns.push((dst2, Searcher::AStar));
        patterns.push((dst1, Searcher::AStar));
    } else {
        let dst1 = last_transition.placement;
        let dst2 = get_nearest_placement_alias(fp.piece(), &dst1, &fp.placement, None);
        patterns.push((dst2, Searcher::HumanOptimized));
        patterns.push((dst2, Searcher::AStar));
        patterns.push((dst1, Searcher::AStar));
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

pub fn search_tspin(piece_t_spec: &PieceSpec, pf: &Playfield, rules: &GameRules) -> Vec<(MoveTransition, LineClear)> {
    debug_assert_eq!(Piece::T, piece_t_spec.piece);
    let move_candidates = get_move_candidates(pf, &FallingPiece::spawn(piece_t_spec, Some(pf)), rules);
    let mut r = vec![];
    for mt in move_candidates.iter() {
        if mt.hint.is_none() {
            continue;
        }
        let line_clear = pf.check_line_clear(
            &FallingPiece::new_with_last_move_transition(piece_t_spec, mt),
            rules.tspin_judgement_mode);
        if line_clear.tspin.is_none() {
            continue;
        }
        r.push((*mt, line_clear));
    }
    r
}

pub fn search_tetris(piece_i_spec: &PieceSpec, pf: &Playfield, rules: &GameRules) -> Option<MoveTransition> {
    debug_assert_eq!(Piece::I, piece_i_spec.piece);
    let move_candidates = get_move_candidates(pf, &FallingPiece::spawn(piece_i_spec, Some(pf)), rules);
    for mt in move_candidates.iter() {
        if mt.placement.orientation.is_even() {
            continue;
        }
        let line_clear = pf.check_line_clear(
            &FallingPiece::new(piece_i_spec, mt.placement),
            rules.tspin_judgement_mode);
        if line_clear.is_tetris() {
            return Some(*mt);
        }
    }
    None
}
