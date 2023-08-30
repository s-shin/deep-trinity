// mod tspin;
// pub mod tree;
// mod memo;
//
// use std::error::Error;
//
// pub mod move_tmpl {
//     use std::collections::HashMap;
//     use serde::{Serialize, Deserialize};
//
//     #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
//     pub enum Piece {
//         I,
//         T,
//         O,
//         J,
//         L,
//         S,
//         Z,
//     }
//
//     impl Piece {
//         pub fn to_core_piece(&self) -> deep_trinity_core::Piece {
//             match self {
//                 Piece::I => deep_trinity_core::Piece::I,
//                 Piece::T => deep_trinity_core::Piece::T,
//                 Piece::O => deep_trinity_core::Piece::O,
//                 Piece::J => deep_trinity_core::Piece::J,
//                 Piece::L => deep_trinity_core::Piece::L,
//                 Piece::S => deep_trinity_core::Piece::S,
//                 Piece::Z => deep_trinity_core::Piece::Z,
//             }
//         }
//     }
//
//     impl From<deep_trinity_core::Piece> for Piece {
//         fn from(p: deep_trinity_core::Piece) -> Self {
//             match p {
//                 deep_trinity_core::Piece::I => Piece::I,
//                 deep_trinity_core::Piece::T => Piece::T,
//                 deep_trinity_core::Piece::O => Piece::O,
//                 deep_trinity_core::Piece::J => Piece::J,
//                 deep_trinity_core::Piece::L => Piece::L,
//                 deep_trinity_core::Piece::S => Piece::S,
//                 deep_trinity_core::Piece::Z => Piece::Z,
//             }
//         }
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     pub struct PieceSequence(String);
//
//     impl PieceSequence {
//         pub fn iter(&self) -> PieceSequenceIterator {
//             PieceSequenceIterator { piece_chars: self.0.chars() }
//         }
//     }
//
//     pub struct PieceSequenceIterator<'a> {
//         piece_chars: std::str::Chars<'a>,
//     }
//
//     impl<'a> Iterator for PieceSequenceIterator<'a> {
//         type Item = deep_trinity_core::Piece;
//
//         fn next(&mut self) -> Option<Self::Item> {
//             if let Some(c) = self.piece_chars.next() {
//                 match deep_trinity_core::Cell::from(c) {
//                     deep_trinity_core::Cell::Block(deep_trinity_core::Block::Piece(p)) => Some(p),
//                     _ => panic!("invalid piece character: {}", c),
//                 }
//             } else {
//                 None
//             }
//         }
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     pub struct Playfield(String);
//
//     impl Playfield {
//         pub fn to_core_playfield(&self) -> deep_trinity_core::Playfield {
//             let mut pf = deep_trinity_core::Playfield::default();
//             for (i, line) in self.0.split('/').enumerate() {
//                 let line = line.replace('_', " ");
//                 pf.set_str_rows((0, i as u8).into(), &[&line]);
//             }
//             pf
//         }
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     #[serde(rename_all = "snake_case")]
//     pub enum TargetState {
//         Opener,
//         Playfield(Playfield),
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     #[serde(rename_all = "snake_case")]
//     pub enum Constraint {
//         PieceOrder(PieceSequence),
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     #[serde(rename_all = "snake_case")]
//     pub enum Action {
//         Hold,
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     pub struct Placement(u8, i8, i8);
//
//     impl Placement {
//         pub fn to_core_placement(&self) -> deep_trinity_core::Placement {
//             deep_trinity_core::Placement::new(deep_trinity_core::Orientation::new(self.0), deep_trinity_core::pos!(self.1, self.2))
//         }
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     #[serde(untagged)]
//     pub enum Move {
//         Action(Action),
//         Placement(Placement),
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     #[serde(rename_all = "snake_case")]
//     pub enum Result {
//         Playfield(Playfield),
//         Error(String),
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     pub struct Example {
//         pub playfield: Playfield,
//         pub pieces: PieceSequence,
//         pub result: Result,
//     }
//
//     #[derive(Debug, Clone, Serialize, Deserialize)]
//     pub struct Template {
//         pub name: String,
//         pub target_state: TargetState,
//         pub constraints: Vec<Constraint>,
//         pub moves: HashMap<Piece, Vec<Move>>,
//         pub examples: Vec<Example>,
//     }
//
//     #[derive(Debug, Clone, Default)]
//     pub struct MoveLoader {
//         cursors: HashMap<Piece, usize>,
//     }
//
//     impl MoveLoader {
//         pub fn next(&mut self, moves: &HashMap<Piece, Vec<Move>>, piece: Piece) -> Option<Move> {
//             let idx = self.cursors.get(&piece).copied().unwrap_or(0);
//             if let Some(mvs) = moves.get(&piece) {
//                 if let Some(mv) = mvs.get(idx) {
//                     self.cursors.insert(piece, idx + 1);
//                     Some(mv.clone())
//                 } else {
//                     None
//                 }
//             } else {
//                 None
//             }
//         }
//     }
// }
//
// fn move_tmpl_test() -> Result<(), Box<dyn Error>> {
//     let args: Vec<String> = std::env::args().collect();
//     let template_file = args.get(1).cloned().unwrap_or("../tmp/templates/tsd_opener_r.yml".into());
//     let data = std::fs::read_to_string(template_file)?;
//     let tmpl: move_tmpl::Template = serde_yaml::from_str(&data)?;
//     println!("{:?}", tmpl);
//
//     let mut game: deep_trinity_core::Game = Default::default();
//     if let Some(ex) = tmpl.examples.get(0) {
//         game.state.playfield = ex.playfield.to_core_playfield();
//         game.state.next_pieces.pieces = ex.pieces.iter().collect();
//     }
//     game.setup_falling_piece(None)?;
//     println!("{}", game);
//     while let Some(_) = game.state.falling_piece {
//         let mut loader: move_tmpl::MoveLoader = Default::default();
//         let piece = game.state.falling_piece.as_ref().unwrap().piece;
//         if let Some(mv) = loader.next(&tmpl.moves, piece.into()) {
//             match mv {
//                 move_tmpl::Move::Placement(placement) => {
//                     let placement = placement.to_core_placement();
//                     let candidates = game.get_move_candidates()?;
//                     let candidates = candidates.iter()
//                         .filter(|mt| mt.placement == placement)
//                         .collect::<Vec<_>>();
//                     let mt = candidates.get(0).unwrap();
//                     let mut fp = game.state.falling_piece.as_mut().unwrap();
//                     fp.placement = mt.placement;
//                     fp.move_path.initial_placement = mt.placement;
//                     if let Some(hint) = mt.hint {
//                         fp.move_path.push(hint);
//                     }
//                     game.lock()?;
//                 }
//                 move_tmpl::Move::Action(move_tmpl::Action::Hold) => {
//                     game.hold()?;
//                 }
//             }
//         }
//         println!("{}", game);
//     }
//
//     Ok(())
// }
//
// fn main() -> Result<(), Box<dyn Error>> {
//     move_tmpl_test()?;
//     Ok(())
// }
