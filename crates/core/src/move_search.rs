use std::collections::HashMap;
use crate::{Playfield, Placement, RotationMode, MovePathItem, MovePath, PieceSpec};

pub mod astar;
pub mod bruteforce;
pub mod humanly_optimized;
pub mod heuristic_bruteforce;

#[derive(Clone)]
pub struct SearchConfiguration<'a> {
    pf: &'a Playfield<'a>,
    piece_spec: &'a PieceSpec<'a>,
    src: Placement,
    mode: RotationMode,
}

impl<'a> SearchConfiguration<'a> {
    pub fn new(pf: &'a Playfield<'a>, piece_spec: &'a PieceSpec<'a>, src: Placement, mode: RotationMode) -> Self {
        Self { pf, piece_spec, src, mode }
    }
}

/// The placement of the kay value is the destination.
/// The placement of the MoveRecordItem of the value is the source.
pub type MoveDestinations = HashMap<Placement, MovePathItem>;

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub src: Placement,
    pub found: MoveDestinations,
}

impl SearchResult {
    pub fn new(src: Placement, found: MoveDestinations) -> Self { Self { src, found } }
    pub fn len(&self) -> usize { self.found.len() }
    pub fn contains(&self, dst: &Placement) -> bool { self.found.contains_key(dst) }
    pub fn get(&self, dst: &Placement) -> Option<MovePath> {
        let mut placement = *dst;
        let mut items: Vec<MovePathItem> = Vec::new();
        let mut i = 0;
        const STOPPER: usize = 10000;
        while let Some(item) = self.found.get(&placement) {
            // if i < 30 { println!("{:?}", item); }
            items.push(MovePathItem::new(item.by, placement));
            placement = item.placement;
            if item.placement == self.src {
                break;
            }
            if i > STOPPER {
                panic!("maybe enter infinite loop");
            }
            i += 1;
        }
        if items.is_empty() {
            return None;
        }
        let mut path = MovePath::new(self.src);
        for item in items.iter().rev() {
            path.merge_or_push(*item);
        }
        Some(path)
    }
}

pub trait MoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult;
}
