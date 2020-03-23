use core::{Game, Placement};

pub trait Bot {
    fn think(&mut self, game: &Game) -> Option<Placement>;
}

pub mod simple;
