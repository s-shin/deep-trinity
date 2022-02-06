use std::collections::HashSet;
use std::error::Error;
use grid::Vec2;
use core::{ORIENTATION_0, ORIENTATION_1, ORIENTATION_2, ORIENTATION_3, Orientation, Piece, Move, MoveTransition, Placement, MovePathItem};
use crate::{Game, Bot, Action};

pub type MoveName = &'static str;

#[derive(Clone, Debug)]
pub struct Opener {
    moves: Vec<(Piece, MoveTransition, MoveName, Vec<MoveName>)>,
    last_move: (Piece, MoveTransition),
}

impl Opener {
    pub fn new(moves: &[(Piece, Orientation, Vec2, MoveName, Vec<MoveName>)], last_move: (Piece, Orientation, Vec2, Option<(Move, Orientation, Vec2)>)) -> Self {
        let moves = moves.iter().map(|(piece, orientation, dst, name, deps)| {
            let mt = MoveTransition::new(Placement::new(*orientation, *dst), None);
            (*piece, mt, *name, deps.clone())
        }).collect::<_>();
        let last_move = (
            last_move.0,
            MoveTransition::new(Placement::new(last_move.1, last_move.2), last_move.3.map(|(m, o, p)| {
                MovePathItem::new(m, Placement::new(o, p))
            })),
        );
        Self { moves, last_move }
    }
}

pub fn tsd_opener_l_01() -> Opener {
    Opener::new(
        &[
            (Piece::I, ORIENTATION_0, Vec2(2, -2), "i", vec![]),
            (Piece::O, ORIENTATION_0, Vec2(7, -1), "", vec![]),
            (Piece::L, ORIENTATION_1, Vec2(-1, 0), "", vec![]),
            (Piece::S, ORIENTATION_1, Vec2(5, 0), "", vec!["i"]),
            (Piece::Z, ORIENTATION_0, Vec2(3, 0), "z", vec!["i"]),
            (Piece::J, ORIENTATION_2, Vec2(3, 2), "", vec!["z"]),
        ],
        (Piece::T, ORIENTATION_2, Vec2(1, 0), Some((Move::Rotate(1), ORIENTATION_1, Vec2(0, 1)))),
    )
}

pub fn tsd_opener_l_02() -> Opener {
    Opener::new(
        &[
            (Piece::I, ORIENTATION_0, Vec2(2, -2), "i", vec![]),
            (Piece::O, ORIENTATION_0, Vec2(7, -1), "", vec![]),
            (Piece::L, ORIENTATION_1, Vec2(-1, 0), "", vec![]),
            (Piece::J, ORIENTATION_2, Vec2(5, 0), "j", vec!["i"]),
            (Piece::S, ORIENTATION_3, Vec2(3, 1), "s", vec!["i"]),
            (Piece::Z, ORIENTATION_0, Vec2(4, 1), "", vec!["j", "s"]),
        ],
        (Piece::T, ORIENTATION_2, Vec2(1, 0), Some((Move::Rotate(1), ORIENTATION_1, Vec2(0, 1)))),
    )
}

pub fn tsd_opener_r_01() -> Opener {
    Opener::new(
        &[
            (Piece::I, ORIENTATION_0, Vec2(2, -2), "i", vec![]),
            (Piece::O, ORIENTATION_0, Vec2(-1, -1), "", vec![]),
            (Piece::J, ORIENTATION_3, Vec2(8, 0), "", vec![]),
            (Piece::Z, ORIENTATION_3, Vec2(2, 0), "", vec!["i"]),
            (Piece::S, ORIENTATION_0, Vec2(4, 0), "s", vec!["i"]),
            (Piece::L, ORIENTATION_2, Vec2(4, 2), "", vec!["s"]),
        ],
        (Piece::T, ORIENTATION_2, Vec2(6, 0), Some((Move::Rotate(-1), ORIENTATION_3, Vec2(7, 1)))),
    )
}

pub fn tsd_opener_r_02() -> Opener {
    Opener::new(
        &[
            (Piece::I, ORIENTATION_0, Vec2(2, -2), "i", vec![]),
            (Piece::O, ORIENTATION_0, Vec2(-1, -1), "", vec![]),
            (Piece::J, ORIENTATION_3, Vec2(8, 0), "", vec![]),
            (Piece::L, ORIENTATION_2, Vec2(2, 0), "l", vec!["i"]),
            (Piece::Z, ORIENTATION_1, Vec2(4, 1), "z", vec!["i"]),
            (Piece::S, ORIENTATION_0, Vec2(3, 1), "", vec!["l", "z"]),
        ],
        (Piece::T, ORIENTATION_2, Vec2(6, 0), Some((Move::Rotate(-1), ORIENTATION_3, Vec2(7, 1)))),
    )
}

#[derive(Clone, Debug)]
struct OpenerMoveDirector {
    moved: Vec<usize>,
    moved_names: HashSet<MoveName>,
    is_end: bool,
}

impl OpenerMoveDirector {
    fn new() -> Self {
        Self { moved: Vec::new(), moved_names: HashSet::new(), is_end: false }
    }
    fn step(&mut self, opener: &Opener, piece: Piece) -> Option<MoveTransition> {
        if self.is_end {
            return None;
        }
        if opener.moves.len() == self.moved.len() {
            return if piece == opener.last_move.0 {
                self.is_end = true;
                Some(opener.last_move.1)
            } else {
                None
            };
        }
        let r = opener.moves.iter().enumerate().find(|(_, (p, _, _, deps))| {
            if piece != *p {
                return false;
            }
            deps.iter().filter(|&name| self.moved_names.contains(name)).count() == deps.len()
        });
        match r {
            Some((i, (_, mt, name, _))) => {
                self.moved.push(i);
                if !name.is_empty() {
                    self.moved_names.insert(*name);
                }
                Some(mt.clone())
            }
            None => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TemplateBot {
    template: Opener,
    director: OpenerMoveDirector,
}

impl TemplateBot {
    pub fn new(template: Opener) -> Self {
        Self { template, director: OpenerMoveDirector::new() }
    }
}

impl Bot for TemplateBot {
    fn think(&mut self, game: &Game) -> Result<Action, Box<dyn Error>> {
        let candidates: HashSet<MoveTransition> = game.get_move_candidates()?;
        if candidates.is_empty() {
            return Err("no move candidates".into());
        }

        let tmpl = &self.template;
        let current_piece = game.state.falling_piece.as_ref().unwrap().piece();
        match self.director.step(tmpl, current_piece) {
            Some(mt) => {
                Ok(Action::Move(mt))
            }
            None => {
                if !self.director.is_end && game.state.can_hold {
                    Ok(Action::Hold)
                } else {
                    Err("end".into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BotRunner, BotRunnerHooks};
    use core::{Cell, Block};

    #[test]
    fn test_template_bot() {
        struct Hooks {
            next_pieces: Vec<Piece>,
        }

        impl BotRunnerHooks for Hooks {
            fn on_start(&mut self, game: &mut Game) -> Result<(), Box<dyn Error>> {
                game.supply_next_pieces(&self.next_pieces);
                game.setup_falling_piece(None)?;
                Ok(())
            }
            fn on_iter(&mut self, game: &mut Game) -> Result<bool, Box<dyn Error>> {
                let fp_exists = game.state.falling_piece.is_some();
                let hold_exists = game.state.hold_piece.is_some();
                if !fp_exists {
                    if game.state.can_hold && hold_exists {
                        game.hold()?;
                        return Ok(true);
                    }
                    return Ok(false);
                }
                Ok(true)
            }
        }

        struct Params {
            pub tmpl: Opener,
            pub next_pieces: Vec<Piece>,
            pub debug_print: bool,
        }

        impl Params {
            fn new(tmpl: Opener, next_pieces_str: &'static str, debug_print: bool) -> Self {
                let next_pieces = next_pieces_str.chars().map(|c| {
                    match Cell::from(c) {
                        Cell::Block(Block::Piece(p)) => p,
                        _ => panic!(),
                    }
                }).collect::<_>();
                Self { tmpl, next_pieces, debug_print }
            }
        }

        for params in &[
            Params::new(tsd_opener_l_01(), "ILOSZJT", false),
            Params::new(tsd_opener_l_02(), "ILOJSZT", false),
            Params::new(tsd_opener_r_01(), "IJOSZLT", false),
            Params::new(tsd_opener_r_02(), "IJOLZST", false),
        ] {
            let runner = BotRunner::new(100, true, None, params.debug_print);
            let mut hooks = Hooks {
                next_pieces: params.next_pieces.clone(),
            };
            let mut bot = TemplateBot::new(params.tmpl.clone());
            runner.run(&mut bot, &mut hooks).unwrap();
        }
    }
}
