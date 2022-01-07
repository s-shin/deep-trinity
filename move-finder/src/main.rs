use core::{Piece, Placement};

type Game = core::Game<'static>;

fn check(mut game: Game, pps: &[PiecePlacement]) -> Vec<PiecePlacement> {
    let r = Vec::with_capacity(pps.len());
    let remains = pps.to_vec();
    while !remains.is_empty() {
        let moves = game.get_move_candidates().unwrap();
        for pp in remains.iter() {
            //
        }
    }
    r
}

macro_rules! pp {
    ($piece_name:ident, $orientation:literal, $x:literal, $y:literal) => {
        PiecePlacement::new(
            core::Piece::$piece_name,
            Placement::new(
                core::ORIENTATIONS[$orientation],
                grid::Vec2($x, $y),
            ),
        )
    }
}

#[derive(Copy, Clone, Debug)]
struct PiecePlacement {
    pub piece: Piece,
    pub placement: Placement,
}

impl PiecePlacement {
    fn new(piece: Piece, placement: Placement) -> Self {
        Self { piece, placement }
    }
}

fn main() {
    let tsd_opener_l_base = [
        pp!(I, 0, 2, -1),
        pp!(O, 0, 7, -1),
        pp!(L, 1, -1, 0),
    ];
    let tsd_opener_l_01 = tsd_opener_l_base.iter().copied().chain([
        pp!(S, 1, 5, 0),
        pp!(Z, 0, 3, 0),
        pp!(J, 2, 3, 2),
        pp!(T, 2, 1, 0),
    ].iter().copied()).collect::<Vec<_>>();

    println!("{:?}", tsd_opener_l_01);
}
