pub use crate::{
    Piece,
    Orientation, ORIENTATION_0, ORIENTATION_1, ORIENTATION_2, ORIENTATION_3,
    Placement,
    MoveTransition,
    RandomPieceGenerator,
};

pub type FallingPiece = crate::FallingPiece<'static>;
pub type Playfield = crate::Playfield<'static>;
pub type Game = crate::Game<'static>;
