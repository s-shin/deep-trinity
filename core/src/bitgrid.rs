// use std::collections::HashMap;
// use std::hash::{Hash, Hasher};
// use std::marker::PhantomData;
// use std::rc::Rc;
// use std::fmt;
// use num_traits::PrimInt;
// use once_cell::sync::Lazy;
// use crate::grid::{Grid, BinaryCell, CellTrait, Vec2, X, Y};
//
// #[derive(Clone, Debug)]
// pub struct PrimBitGridConstants<Int: PrimInt> {
//     pub num_bits: u32,
//     pub stride: X,
//     pub width: X,
//     pub max_height: Y,
//     pub height: Y,
//     pub cells_mask: Int,
//     row_masks: Vec<Int>,
//     col_masks: Vec<Int>,
//     lhs_cols_masks: Vec<Int>,
//     rhs_cols_masks: Vec<Int>,
//     bottom_side_rows_masks: Vec<Int>,
//     top_side_rows_masks: Vec<Int>,
// }
//
// impl<Int: PrimInt> PrimBitGridConstants<Int> {
//     pub fn new(width: X, height: Option<Y>, stride: Option<X>) -> Self {
//         let num_bits = Int::zero().count_zeros();
//         let stride = stride.unwrap_or(width);
//         assert!(width <= stride);
//         assert!(1 <= stride && stride as u32 <= num_bits);
//         let max_height = (num_bits / stride as u32) as Y;
//         let height = height.unwrap_or(max_height);
//         assert!(1 <= height && height <= max_height);
//         let row_masks = {
//             let m = (!Int::zero()).unsigned_shr(num_bits - width as u32);
//             (0..height).map(|y| m << (y as usize * stride)).collect::<Vec<_>>()
//         };
//         let col_masks = {
//             let mut m = Int::zero();
//             for y in 0..height {
//                 m = m | Int::one() << (y as usize * stride);
//             }
//             (0..width).map(|x| m << x as usize).collect::<Vec<_>>()
//         };
//         let cells_mask = {
//             let mut m = Int::zero();
//             for y in 0..height {
//                 m = m | row_masks[y as usize];
//             }
//             m
//         };
//         let lhs_cols_masks = {
//             let w = width as usize;
//             let mut r = Vec::with_capacity(w);
//             let mut t = Int::zero();
//             for x in 0..(w - 1) {
//                 t = t | col_masks[x];
//                 r.push(t);
//             }
//             r
//         };
//         let rhs_cols_masks = {
//             let w = width as usize;
//             let mut r = Vec::with_capacity(w);
//             let mut t = Int::zero();
//             for x in (1..w).rev() {
//                 t = t | col_masks[x];
//                 r.push(t);
//             }
//             r
//         };
//         let bottom_side_rows_masks = {
//             let h = height as usize;
//             let mut r = Vec::with_capacity(h);
//             let mut t = Int::zero();
//             for y in 0..(h - 1) {
//                 t = t | row_masks[y];
//                 r.push(t);
//             }
//             r
//         };
//         let top_side_rows_masks = {
//             let h = height as usize;
//             let mut r = Vec::with_capacity(h);
//             let mut t = Int::zero();
//             for y in (1..h).rev() {
//                 t = t | row_masks[y];
//                 r.push(t);
//             }
//             r
//         };
//         Self {
//             num_bits,
//             stride,
//             width,
//             max_height,
//             height,
//             cells_mask,
//             row_masks,
//             col_masks,
//             lhs_cols_masks,
//             rhs_cols_masks,
//             bottom_side_rows_masks,
//             top_side_rows_masks,
//         }
//     }
//     pub fn row_mask(&self, y: Y) -> Int {
//         debug_assert!(0 <= y && y < self.height);
//         self.row_masks[y as usize]
//     }
//     pub fn col_mask(&self, x: X) -> Int {
//         debug_assert!(0 <= x && x < self.width);
//         self.col_masks[x as usize]
//     }
//     pub fn lhs_cols_mask(&self, num_blocks: X) -> Int {
//         let n = num_blocks;
//         debug_assert!(0 < n && n < self.width - 1);
//         self.lhs_cols_masks[(n - 1) as usize]
//     }
//     pub fn rhs_cols_mask(&self, num_blocks: X) -> Int {
//         let n = num_blocks;
//         debug_assert!(0 < n && n < self.width - 1);
//         self.rhs_cols_masks[(n - 1) as usize]
//     }
//     pub fn lhs_empty_cols_mask(&self, num_empty_cells: X) -> Int {
//         let n = num_empty_cells;
//         debug_assert!(0 < n && n < self.width - 1);
//         self.rhs_cols_masks[((self.width - 1) - n) as usize]
//     }
//     pub fn rhs_empty_cols_mask(&self, num_empty_cells: X) -> Int {
//         let n = num_empty_cells;
//         debug_assert!(0 < n && n < self.width - 1);
//         self.lhs_cols_masks[((self.width - 1) - n) as usize]
//     }
//     pub fn top_side_rows_mask(&self, num_blocks: Y) -> Int {
//         let n = num_blocks;
//         debug_assert!(0 < n && n < self.height - 1);
//         self.top_side_rows_masks[(n - 1) as usize]
//     }
//     pub fn bottom_side_rows_mask(&self, num_blocks: Y) -> Int {
//         let n = num_blocks;
//         debug_assert!(0 < n && n < self.height - 1);
//         self.bottom_side_rows_masks[(n - 1) as usize]
//     }
//     pub fn top_side_empty_rows_mask(&self, num_empty_cells: Y) -> Int {
//         let n = num_empty_cells;
//         debug_assert!(0 < n && n < self.height - 1);
//         self.bottom_side_rows_masks[((self.height - 1) - n) as usize]
//     }
//     pub fn bottom_side_empty_rows_mask(&self, num_empty_cells: Y) -> Int {
//         let n = num_empty_cells;
//         debug_assert!(0 < n && n < self.height - 1);
//         self.top_side_rows_masks[((self.height - 1) - n) as usize]
//     }
// }
//
// #[derive(Clone, Debug)]
// pub struct PrimBitGrid<'a, Int: PrimInt, C: CellTrait = BinaryCell> {
//     constants: &'a PrimBitGridConstants<Int>,
//     cells: Int,
//     phantom: PhantomData<fn() -> C>,
// }
//
// impl<'a, Int: PrimInt, C: CellTrait> PrimBitGrid<'a, Int, C> {
//     pub fn new(constants: &'a PrimBitGridConstants<Int>) -> Self {
//         Self { constants, cells: Int::zero(), phantom: PhantomData }
//     }
//     pub fn constants(&self) -> &'a PrimBitGridConstants<Int> { self.constants }
//     fn bit_index(&self, pos: Vec2) -> u32 { (self.constants.width * pos.1 + pos.0) as u32 }
//     fn cell_mask(&self, pos: Vec2) -> Int { Int::one() << self.bit_index(pos) as usize }
//     fn put_same_stride<OtherInt: PrimInt, OtherCell: CellTrait>(&mut self, pos: Vec2, other: &PrimBitGrid<OtherInt, OtherCell>) {
//         assert!(self.constants.num_bits >= other.constants.num_bits);
//         assert_eq!(self.constants.stride, other.constants.stride);
//         let Vec2(x, y) = pos;
//         let other_cells = {
//             let cells = Int::from(other.cells);
//             if x == 0 {
//                 cells
//             } else if x > 0 {
//                 cells & self.constants.rhs_empty_cols_mask(x) << x as usize
//             } else {
//                 (cells & self.constants.lhs_empty_cols_mask(-x)).unsigned_shr(-x as u32)
//             }
//         };
//         self.cells = self.cells |
//             if y == 0 {
//                 other_cells
//             } else if y > 0 {
//                 (other_cells << (self.constants.stride * y) as usize) & self.constants.cells_mask
//             } else {
//                 other_cells.unsigned_shr((self.constants.stride * -y) as u32)
//             };
//     }
//     fn can_put_same_stride<OtherInt: PrimInt, OtherCell: CellTrait>(&self, pos: Vec2, other: &PrimBitGrid<OtherInt, OtherCell>) -> bool {
//         assert!(self.constants.num_bits >= other.constants.num_bits);
//         assert_eq!(self.constants.stride, other.constants.stride);
//         let Vec2(x, y) = pos;
//         let other_cells = {
//             let cells = Int::from(other.cells);
//             if x == 0 {
//                 cells
//             } else if x > 0 {
//                 if cells & self.constants.rhs_cols_mask(x) != Int::zero() {
//                     return false;
//                 }
//                 cells << x as usize
//             } else {
//                 if cells & self.constants.lhs_cols_mask(-x) != Int::zero() {
//                     return false;
//                 }
//                 cells.unsigned_shr(-x as u32)
//             }
//         };
//         self.cells & if y == 0 {
//             other_cells
//         } else if y > 0 {
//             if other_cells & self.constants.top_side_rows_mask(y) != Int::zero() {
//                 return false;
//             }
//             other_cells << (self.constants.stride * y) as usize
//         } else {
//             if other_cells & self.constants.bottom_side_rows_mask(-y) != Int::zero() {
//                 return false;
//             }
//             other_cells.unsigned_shr((self.constants.stride * -y) as u32)
//         } == Int::zero()
//     }
//     pub fn put_fast<OtherInt: PrimInt, OtherCell: CellTrait>(&mut self, pos: Vec2, other: &PrimBitGrid<OtherInt, OtherCell>) {
//         if self.constants.num_bits >= other.constants.num_bits && self.constants.stride == other.constants.stride {
//             return self.put_same_stride(pos, other);
//         }
//         todo!()
//     }
//     pub fn can_put_fast(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> bool {
//         if self.constants.num_bits >= other.constants.num_bits && self.constants.stride == other.constants.stride {
//             return self.can_put_same_stride(pos, other);
//         }
//         todo!()
//     }
//     pub fn num_droppable_rows_fast(&self, _pos: Vec2, _sub: &PrimBitGrid<Int, C>) -> Y {
//         todo!()
//     }
//     pub fn swap_row_with_other<OtherCell: CellTrait>(&mut self, y: Y, other: &mut PrimBitGrid<Int, OtherCell>, other_y: Y) {
//         assert_eq!(self.constants.stride, other.constants.stride);
//         assert_eq!(self.constants.width, other.constants.width);
//         assert!(0 <= y && y < self.constants.height);
//         assert!(0 <= other_y && other_y < other.constants.height);
//         let stride = self.constants.stride as u32;
//         let mask = self.constants.row_mask(y);
//         let row = self.cells & mask;
//         let other_mask = other.constants.row_mask(other_y);
//         let other_row = other.cells & other_mask;
//         self.cells = (self.cells & !mask) | if y == other_y {
//             other_row
//         } else if y > other_y {
//             other_row << ((y - other_y) as usize * stride)
//         } else {
//             other_row.unsigned_shr((other_y - y) as u32 * stride);
//         };
//         other.cells = (other.cells & !other_mask) | if y == other_y {
//             row
//         } else if other_y > y {
//             row << ((other_y - y) as usize * stride)
//         } else {
//             row.unsigned_shr((y - other_y) as u32 * stride)
//         };
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> Grid<C> for PrimBitGrid<'_, Int, C> {
//     fn width(&self) -> X { self.constants.width }
//     fn height(&self) -> Y { self.constants.height }
//     fn cell(&self, pos: Vec2) -> C {
//         if self.cells & self.cell_mask(pos) == Int::zero() { C::empty() } else { C::any_block() }
//     }
//     fn set_cell(&mut self, pos: Vec2, cell: C) {
//         let m = self.cell_mask(pos);
//         if cell.is_empty() {
//             self.cells = self.cells & !m;
//         } else {
//             self.cells = self.cells | m;
//         }
//     }
//     fn fill_row(&mut self, y: Y, cell: C) {
//         let m = self.constants.row_mask(y);
//         if cell.is_empty() {
//             self.cells = self.cells & !m;
//         } else {
//             self.cells = self.cells | m;
//         }
//     }
//     fn fill_all(&mut self, cell: C) {
//         self.cells = if cell.is_empty() { Int::zero() } else { self.constants.cells_mask };
//     }
//     fn is_row_filled(&self, y: Y) -> bool {
//         let m = self.constants.row_mask(y);
//         self.cells & m == m
//     }
//     fn is_row_empty(&self, y: Y) -> bool {
//         let m = self.constants.row_mask(y);
//         self.cells & m == Int::zero()
//     }
//     fn is_col_filled(&self, x: X) -> bool {
//         let m = self.constants.col_mask(x);
//         self.cells & m == m
//     }
//     fn is_col_empty(&self, x: X) -> bool {
//         let m = self.constants.col_mask(x);
//         self.cells & m == Int::zero()
//     }
//     fn swap_rows(&mut self, mut y1: Y, mut y2: Y) {
//         if y1 == y2 {
//             return;
//         }
//         if y1 > y2 {
//             std::mem::swap(&mut y1, &mut y2);
//         }
//         let dy = (y2 - y1) as usize;
//         debug_assert!(dy > 0);
//         let dy_shift = dy * self.width() as usize;
//         let m1 = self.constants.row_mask(y1);
//         let m2 = self.constants.row_mask(y2);
//         self.cells = (self.cells & !m1 & !m2) | (self.cells & m1) << dy_shift | (self.cells & m2) >> dy_shift;
//     }
//     fn num_blocks_of_row(&self, y: Y) -> usize {
//         let m = self.constants.row_mask(y);
//         (self.cells & m).count_ones() as usize
//     }
//     fn num_blocks(&self) -> usize {
//         debug_assert!(self.cells & !self.constants.cells_mask == Int::zero());
//         self.cells.count_ones() as usize
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> fmt::Display for PrimBitGrid<'_, Int, C> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
// }
//
// impl<Int: PrimInt, C: CellTrait> PartialEq for PrimBitGrid<'_, Int, C> {
//     fn eq(&self, other: &Self) -> bool {
//         self.size() == other.size() && self.cells == other.cells
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> Eq for PrimBitGrid<'_, Int, C> {}
//
// impl<Int: PrimInt + Hash, C: CellTrait> Hash for PrimBitGrid<'_, Int, C> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.size().hash(state);
//         self.cells.hash(state);
//     }
// }
//
// //---
//
// #[derive(Clone)]
// pub struct BitGrid<'a, Int: PrimInt, C: CellTrait = BinaryCell> {
//     size: Vec2,
//     prim_grids: Vec<PrimBitGrid<'a, Int, C>>,
// }
//
// impl<'a, Int: PrimInt, C: CellTrait> BitGrid<'a, Int, C> {
//     pub fn new(repeated: &'a PrimBitGridConstants<Int>, n: Y, edge: Option<&'a PrimBitGridConstants<Int>>) -> Self {
//         debug_assert!(n >= 0);
//         debug_assert!(edge.is_none() || repeated.width == edge.unwrap().width);
//         let size = Vec2(repeated.width, repeated.height * n + edge.map_or(0, |c| c.height));
//         let num_prim_grids = (n + edge.map_or(0, |_| 1)) as usize;
//         let mut prim_grids = Vec::with_capacity(num_prim_grids);
//         for _ in 0..n {
//             prim_grids.push(PrimBitGrid::new(repeated));
//         }
//         if let Some(c) = edge {
//             prim_grids.push(PrimBitGrid::new(c));
//         }
//         debug_assert_eq!(num_prim_grids, prim_grids.len());
//         Self { size, prim_grids }
//     }
//     pub fn prim_height(&self) -> Y { self.prim_grids.first().unwrap().height() }
//     pub fn prim_grid_index(&self, y: Y) -> (usize, Y) {
//         let h = self.prim_height();
//         ((y / h) as usize, y % h)
//     }
//     pub fn put_same_stride_prim<OtherInt: PrimInt, OtherCell: CellTrait>(&mut self, pos: Vec2, other: &PrimBitGrid<OtherInt, OtherCell>) {
//         let (i, y) = self.prim_grid_index(pos.1);
//         let can_put = self.prim_grids.get(i).unwrap().can_put_same_stride((pos.0, y).into(), other);
//         self.prim_grids.get_mut(i).unwrap().put_same_stride((pos.0, y).into(), other);
//         if !can_put && i + 1 < self.prim_grids.len() {
//             self.prim_grids.get_mut(i + 1).unwrap().put_same_stride((pos.0, y - self.prim_height()).into(), other);
//         }
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> Grid<C> for BitGrid<'_, Int, C> {
//     fn width(&self) -> X { self.size.0 }
//     fn height(&self) -> Y { self.size.1 }
//     fn cell(&self, pos: Vec2) -> C {
//         let (i, y) = self.prim_grid_index(pos.1);
//         self.prim_grids.get(i).unwrap().cell((pos.0, y).into())
//     }
//     fn set_cell(&mut self, pos: Vec2, cell: C) {
//         let (i, y) = self.prim_grid_index(pos.1);
//         self.prim_grids.get_mut(i).unwrap().set_cell((pos.0, y).into(), cell);
//     }
//     fn fill_row(&mut self, y: Y, cell: C) {
//         let (i, y) = self.prim_grid_index(y);
//         self.prim_grids.get_mut(i).unwrap().fill_row(y, cell);
//     }
//     fn fill_all(&mut self, cell: C) {
//         for g in self.prim_grids.iter_mut() {
//             g.fill_all(cell);
//         }
//     }
//     fn is_row_filled(&self, y: Y) -> bool {
//         let (i, y) = self.prim_grid_index(y);
//         self.prim_grids.get(i).unwrap().is_row_filled(y)
//     }
//     fn is_row_empty(&self, y: Y) -> bool {
//         let (i, y) = self.prim_grid_index(y);
//         self.prim_grids.get(i).unwrap().is_row_empty(y)
//     }
//     fn is_col_filled(&self, x: X) -> bool {
//         for g in self.prim_grids.iter() {
//             if !g.is_col_filled(x) {
//                 return false;
//             }
//         }
//         true
//     }
//     fn is_col_empty(&self, x: X) -> bool {
//         for g in self.prim_grids.iter() {
//             if !g.is_col_empty(x) {
//                 return false;
//             }
//         }
//         true
//     }
//     fn swap_rows(&mut self, mut y1: Y, mut y2: Y) {
//         if y1 == y2 {
//             return;
//         }
//         if y1 > y2 {
//             std::mem::swap(&mut y1, &mut y2);
//         }
//         let (i1, y1) = self.prim_grid_index(y1);
//         let (i2, y2) = self.prim_grid_index(y2);
//         if i1 == i2 {
//             self.prim_grids.get_mut(i1).unwrap().swap_rows(y1, y2);
//         } else {
//             debug_assert!(i1 < i2);
//             let (left, right) = self.prim_grids.split_at_mut(i1 + 1);
//             let g1 = left.get_mut(0).unwrap();
//             let g2 = right.get_mut(i2 - i1 - 1).unwrap();
//             g1.swap_row_with_same_width(y1, g2, y2);
//         }
//     }
//     fn num_blocks_of_row(&self, y: Y) -> usize {
//         let (i, y) = self.prim_grid_index(y);
//         self.prim_grids.get(i).unwrap().num_blocks_of_row(y)
//     }
//     fn num_blocks(&self) -> usize {
//         let mut n = 0;
//         for g in self.prim_grids.iter() {
//             n += g.num_blocks();
//         }
//         n
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> fmt::Display for BitGrid<'_, Int, C> {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
// }
//
// impl<Int: PrimInt, C: CellTrait> PartialEq for BitGrid<'_, Int, C> {
//     fn eq(&self, other: &Self) -> bool {
//         self.size() == other.size() && self.prim_grids == other.prim_grids
//     }
// }
//
// impl<Int: PrimInt, C: CellTrait> Eq for BitGrid<'_, Int, C> {}
//
// impl<Int: PrimInt + Hash, C: CellTrait> Hash for BitGrid<'_, Int, C> {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.size().hash(state);
//         self.prim_grids.hash(state);
//     }
// }
//
// //---
//
// pub fn prim_bit_grid_constants_u64_w10(height: Y) -> &'static PrimBitGridConstants<u64> {
//     static CONSTANTS_LIST: Lazy<Vec<PrimBitGridConstants<u64>>> = Lazy::new(|| {
//         (0..6).map(|i| PrimBitGridConstants::new(10, Some((i + 1) as Y)))
//             .collect()
//     });
//     CONSTANTS_LIST.get((height - 1) as usize).unwrap()
// }
//
// pub fn new_bit_grid_u64_10x40<C: CellTrait>() -> BitGrid<'static, u64, C> {
//     BitGrid::new(prim_bit_grid_constants_u64_w10(6), 6, Some(prim_bit_grid_constants_u64_w10(4)))
// }
//
// //---
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::grid::{CellTrait, Grid, TestHelper};
//
//     #[test]
//     fn test_prim_bit_grid_constants() {
//         let c = PrimBitGridConstants::<u32>::new(10, None);
//
//         assert_eq!(32, c.num_bits);
//         assert_eq!(10, c.width);
//         assert_eq!(3, c.max_height);
//         assert_eq!(3, c.height);
//
//         assert_eq!(&[
//             0b1111111111,
//             0b1111111111_0000000000,
//             0b1111111111_0000000000_0000000000,
//         ], c.row_masks.as_slice());
//
//         assert_eq!(&[
//             0b0000000001_0000000001_0000000001,
//             0b0000000010_0000000010_0000000010,
//             0b0000000100_0000000100_0000000100,
//             0b0000001000_0000001000_0000001000,
//             0b0000010000_0000010000_0000010000,
//             0b0000100000_0000100000_0000100000,
//             0b0001000000_0001000000_0001000000,
//             0b0010000000_0010000000_0010000000,
//             0b0100000000_0100000000_0100000000,
//             0b1000000000_1000000000_1000000000,
//         ], c.col_masks.as_slice());
//
//         assert_eq!(0b1111111111_1111111111_1111111111, c.cells_mask);
//
//         assert_eq!(&[
//             0b0000000001_0000000001_0000000001,
//             0b0000000011_0000000011_0000000011,
//             0b0000000111_0000000111_0000000111,
//             0b0000001111_0000001111_0000001111,
//             0b0000011111_0000011111_0000011111,
//             0b0000111111_0000111111_0000111111,
//             0b0001111111_0001111111_0001111111,
//             0b0011111111_0011111111_0011111111,
//             0b0111111111_0111111111_0111111111,
//         ], c.lhs_cols_masks.as_slice());
//
//         assert_eq!(&[
//             0b1000000000_1000000000_1000000000,
//             0b1100000000_1100000000_1100000000,
//             0b1110000000_1110000000_1110000000,
//             0b1111000000_1111000000_1111000000,
//             0b1111100000_1111100000_1111100000,
//             0b1111110000_1111110000_1111110000,
//             0b1111111000_1111111000_1111111000,
//             0b1111111100_1111111100_1111111100,
//             0b1111111110_1111111110_1111111110,
//         ], c.rhs_cols_masks.as_slice());
//
//         assert_eq!(&[
//             0b0000000000_0000000000_1111111111,
//             0b0000000000_1111111111_1111111111,
//         ], c.bottom_side_rows_masks.as_slice());
//
//         assert_eq!(&[
//             0b1111111111_0000000000_0000000000,
//             0b1111111111_1111111111_0000000000,
//         ], c.top_side_rows_masks.as_slice());
//     }
//
//     #[test]
//     fn test_prim_bit_grid_basic() {
//         let helper = TestHelper::new(|| PrimBitGrid::<_>::new(prim_bit_grid_constants_u64_w10(6)));
//         helper.basic();
//     }
//
//     #[test]
//     fn test_prim_big_grid_put_fast() {
//         let mut g1 = PrimBitGrid::<_>::new(prim_bit_grid_constants_u64_w10(6));
//         let mut g2 = g1.clone();
//         g2.set_rows_with_strs((1, 1).into(), &[
//             " @@ ",
//             "@ @@",
//             "@@ @",
//             " @@ ",
//         ]);
//         g1.put_fast((-2, -2).into(), &g2);
//         assert_eq!(6, g1.num_blocks());
//         let mut block_positions: Vec<(i8, i8)> = vec![
//             (0, 0), (0, 2), (1, 1), (1, 2), (2, 0), (2, 1),
//         ];
//         for pos in &block_positions {
//             assert!(g1.cell((*pos).into()).is_block());
//         }
//         g1.put_fast((6, 2).into(), &g2);
//         assert_eq!(12, g1.num_blocks());
//         block_positions.extend([
//             (7, 4), (7, 5), (8, 3), (8, 4), (9, 3), (9, 5),
//         ].iter());
//         for pos in &block_positions {
//             assert!(g1.cell((*pos).into()).is_block());
//         }
//     }
//
//     #[test]
//     fn test_prim_big_grid_can_put_fast() {
//         let mut g1 = PrimBitGrid::<_>::new(prim_bit_grid_constants_u64_w10(6));
//         let mut g2 = g1.clone();
//         g2.set_rows_with_strs((1, 1).into(), &[
//             " @@ ",
//             "@ @@",
//             "@@ @",
//             " @@ ",
//         ]);
//         assert!(g1.can_put_fast((-1, -1).into(), &g2));
//         assert!(!g1.can_put_fast((-2, 0).into(), &g2));
//         assert!(!g1.can_put_fast((0, -2).into(), &g2));
//         g1.put_fast((-1, -1).into(), &g2);
//         assert!(!g1.can_put_fast((-1, -1).into(), &g2));
//     }
//
//     #[test]
//     fn test_bit_grid_basic() {
//         let c10x3 = PrimBitGridConstants::<u32>::new(10, Some(3));
//         let c10x1 = PrimBitGridConstants::<u32>::new(10, Some(1));
//         let helper = TestHelper::new(|| BitGrid::<u32>::new(&c10x3, 3, Some(&c10x1)));
//         helper.basic();
//     }
// }
