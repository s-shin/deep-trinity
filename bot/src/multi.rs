// use core::{MoveTransition, Placement, Piece, Game};
// use std::error::Error;
// use crate::Bot;
//
// #[derive(Debug, Clone, Copy)]
// enum PlacementConstraint {
//     XRange(i8, i8),
//     NotXRange(i8, i8),
// }
//
// #[derive(Debug, Clone)]
// enum Action {
//     MoveTo(Placement),
//     MoveToAny(Vec<PlacementConstraint>),
//     Hold,
// }
//
// #[derive(Debug, Clone)]
// struct ActionDirection {
//     actions: [Vec<Action>; 7],
// }
//
// impl ActionDirection {
//     fn new(s: Vec<Action>, z: Vec<Action>, l: Vec<Action>, j: Vec<Action>, i: Vec<Action>, t: Vec<Action>, o: Vec<Action>) -> Self {
//         Self { actions: [s, z, l, j, i, t, o] }
//     }
//     fn get(&self, p: Piece) -> Option<&Vec<Action>> {
//         self.actions.iter().nth(p.to_usize())
//     }
// }
//
// struct ActionDirector {
//     direction: ActionDirection,
//     cursors: [usize; 7],
// }
//
// impl ActionDirector {
//     pub fn next(&mut self, piece: Piece) -> Option<Action> {
//         let idx: usize = self.cursors[piece.to_usize()];
//         if let Some(actions) = self.direction.get(piece.into()) {
//             if let Some(action) = actions.get(idx) {
//                 self.cursor[piece.to_usize()] = idx + 1;
//                 Some(action.clone())
//             } else {
//                 None
//             }
//         } else {
//             None
//         }
//     }
// }
//
// // struct Strategy {
// //     action_direction: ActionDirection,
// //     //
// // }
// //
// // trait StrategyDetector {
// //     fn detect(&mut self, game: &Game) -> Result<Vec<Strategy>, Box<dyn Error>>;
// // }
// //
// // fn collect_strategies(game: &Game, detectors: &mut [Box<dyn StrategyDetector>]) -> Vec<Strategy> {
// //     detectors.iter_mut()
// //         .map(|d| {
// //             match d.detect(game) {
// //                 Ok(r) => r,
// //                 Err(e) => {
// //                     println!("WARNING: {}", e);
// //                     vec![]
// //                 }
// //             }
// //         })
// //         .flatten()
// //         .collect::<Vec<_>>()
// // }
// //
// // //---
// //
// //
// //
// // //---
// //
// // struct MultiBot {
// //     detectors: Vec<Box<dyn StrategyDetector>>,
// //     director: Option<ActionDirector>,
// // }
// //
// // impl Bot for MultiBot {
// //     fn think(&mut self, game: &Game) -> Result<crate::Action, Box<dyn Error>> {
// //         let _strategies = collect_strategies(game, &mut self.detectors);
// //         panic!("TODO");
// //     }
// // }
