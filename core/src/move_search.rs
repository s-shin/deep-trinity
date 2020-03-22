use std::collections::HashMap;
use crate::{Playfield, Piece, Placement, RotationMode, MoveRecordItem, MoveRecord};

pub mod astar;
pub mod bruteforce;
pub mod humanly_optimized;

pub struct SearchConfiguration<'a> {
    pf: &'a Playfield,
    piece: Piece,
    src: Placement,
    mode: RotationMode,
}

impl<'a> SearchConfiguration<'a> {
    pub fn new(pf: &'a Playfield, piece: Piece, src: Placement, mode: RotationMode) -> Self {
        Self { pf, piece, src, mode }
    }
}

/// The placement of MoveRecordItem is **source** one.
pub type MoveDestinations = HashMap<Placement, MoveRecordItem>;

#[derive(Clone, Debug)]
pub struct SearchResult {
    pub src: Placement,
    pub found: MoveDestinations,
}

impl SearchResult {
    pub fn new(src: Placement, found: MoveDestinations) -> Self { Self { src, found } }
    pub fn len(&self) -> usize { self.found.len() }
    pub fn contains(&self, dst: &Placement) -> bool { self.found.contains_key(dst) }
    pub fn get(&self, dst: &Placement) -> Option<MoveRecord> {
        let mut placement = *dst;
        let mut items: Vec<MoveRecordItem> = Vec::new();
        while let Some(item) = self.found.get(&placement) {
            items.push(MoveRecordItem::new(item.by, placement));
            placement = item.placement;
            if item.placement == self.src {
                break;
            }
        }
        if items.is_empty() {
            return None;
        }
        let mut record = MoveRecord::new(self.src);
        for item in items.iter().rev() {
            record.merge_or_push(*item);
        }
        Some(record)
    }
}

pub trait MoveSearcher {
    fn search(&mut self, conf: &SearchConfiguration) -> SearchResult;
}
