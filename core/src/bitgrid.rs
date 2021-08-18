use super::grid::{Grid, BitCell, CellTrait, Vec2, X, Y};
use once_cell::sync::Lazy;
use num_traits::PrimInt;

fn gen_prim_bit_grid_10_row_masks<Int: PrimInt>() -> Vec<Int> {
    let height = (Int::zero().count_zeros() / 10) as usize;
    let m = Int::from(0b1111111111).unwrap();
    (0..height).map(|y| m << y).collect::<Vec<_>>()
}

fn gen_prim_bit_grid_10_col_masks<Int: PrimInt>() -> Vec<Int> {
    let height = (Int::zero().count_zeros() / 10) as usize;
    let mut m = Int::from(0b0000000001).unwrap();
    for _i in 0..height {
        m = m | m << 1;
    }
    (0..10).map(|y| m << y).collect::<Vec<_>>()
}

struct BitGrid10Constants<Int: PrimInt> {
    row_masks: Vec<Int>,
    col_masks: Vec<Int>,
    cells_mask: Int,
}

impl<Int: PrimInt> BitGrid10Constants<Int> {
    fn new() -> Self {
        let num_bits = Int::zero().count_zeros() as usize;
        let num_unused_bits = num_bits - num_bits % 10;
        let cells_mask = ((!Int::zero()) << num_unused_bits) >> num_unused_bits;
        Self {
            row_masks: gen_prim_bit_grid_10_row_masks(),
            col_masks: gen_prim_bit_grid_10_col_masks(),
            cells_mask,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BitGrid10<Int: PrimInt + 'static> {
    height: Y,
    cells: Int,
    constants: &'static BitGrid10Constants<Int>,
}

impl<Int: PrimInt> BitGrid10<Int> {
    fn new(constants: &'static BitGrid10Constants<Int>) -> Self {
        let num_bits = Int::zero().count_zeros();
        let height = (num_bits / 10) as Y;
        debug_assert!(height > 0);
        Self {
            height,
            cells: Int::zero(),
            constants,
        }
    }
    fn bit_index(&self, pos: Vec2) -> u8 {
        debug_assert!(self.is_inside(pos));
        (pos.0 * pos.1 * self.width()) as u8
    }
    fn cell_mask(&self, pos: Vec2) -> Int {
        Int::one() << self.bit_index(pos) as usize
    }
    fn row_mask(&self, y: Y) -> Int {
        debug_assert!(0 <= y && y < self.height());
        self.constants.row_masks[y as usize]
    }
    fn col_mask(&self, x: X) -> Int {
        debug_assert!(0 <= x && x < self.width());
        self.constants.col_masks[x as usize]
    }
    pub fn put_fast<I: PrimInt>(&mut self, pos: Vec2, sub: &BitGrid10<I>) {
        debug_assert!(self.is_inside(pos));
        let sub_cells = Int::from(sub.cells).unwrap();
        if pos == (0, 0).into() {
            self.cells = self.cells | sub_cells;
        } else {
            self.cells = self.cells | (sub_cells << (self.bit_index(pos) as usize + 1) & self.constants.cells_mask);
        }
    }
    pub fn can_put_fast<I: PrimInt>(&self, pos: Vec2, sub: &BitGrid10<I>) -> bool {
        debug_assert!(self.is_inside(pos));
        let sub_cells = Int::from(sub.cells).unwrap();
        if pos == (0, 0).into() {
            self.cells & sub_cells == Int::zero()
        } else {
            self.cells & (sub_cells << (self.bit_index(pos) as usize + 1) & self.constants.cells_mask) == Int::zero()
        }
    }
}

impl<Int: PrimInt> Grid<BitCell> for BitGrid10<Int> {
    fn width(&self) -> X { 10 }
    fn height(&self) -> Y { self.height }
    fn cell(&self, pos: Vec2) -> BitCell {
        if self.cells & self.cell_mask(pos) == Int::zero() {
            BitCell::empty()
        } else {
            BitCell::any_block()
        }
    }
    fn set_cell(&mut self, pos: Vec2, cell: BitCell) {
        let m = self.cell_mask(pos);
        if cell.is_empty() {
            self.cells = self.cells & !m;
        } else {
            self.cells = self.cells | m;
        }
    }
    fn fill_row(&mut self, y: Y, cell: BitCell) {
        let m = self.row_mask(y);
        if cell.is_empty() {
            self.cells = self.cells & !m;
        } else {
            self.cells = self.cells | m;
        }
    }
    fn is_row_filled(&self, y: Y) -> bool {
        let m = self.row_mask(y);
        self.cells & m == m
    }
    fn is_row_empty(&self, y: Y) -> bool { self.cells & self.row_mask(y) == Int::zero() }
    fn is_col_filled(&self, x: X) -> bool {
        let m = self.col_mask(x);
        self.cells & m == m
    }
    fn is_col_empty(&self, x: X) -> bool { self.cells & self.col_mask(x) == Int::zero() }
    fn swap_rows(&mut self, mut y1: Y, mut y2: Y) {
        if y1 == y2 {
            return;
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }
        let dy = (y2 - y1) as usize;
        debug_assert!(dy > 0);
        let dy_shift = dy * self.width() as usize;
        let m1 = self.row_mask(y1);
        let m2 = self.row_mask(y2);
        self.cells = (self.cells & !m1 & !m2) | (self.cells & m1) << dy_shift | (self.cells & m2) >> dy_shift;
    }
    fn num_blocks_of_row(&self, y: Y) -> usize { (self.cells & self.row_mask(y)).count_ones() as usize }
    fn num_blocks(&self) -> usize { self.cells.count_ones() as usize }
}

pub fn new_bit_grid_10_12() -> BitGrid10<u128> {
    static CONSTANTS: Lazy<BitGrid10Constants<u128>> = Lazy::new(|| BitGrid10Constants::new());
    BitGrid10::new(&CONSTANTS)
}

pub fn new_bit_grid_10_6() -> BitGrid10<u64> {
    static CONSTANTS: Lazy<BitGrid10Constants<u64>> = Lazy::new(|| BitGrid10Constants::new());
    BitGrid10::new(&CONSTANTS)
}

pub fn new_bit_grid_10_3() -> BitGrid10<u32> {
    static CONSTANTS: Lazy<BitGrid10Constants<u32>> = Lazy::new(|| BitGrid10Constants::new());
    BitGrid10::new(&CONSTANTS)
}

pub fn new_bit_grid_10_1() -> BitGrid10<u16> {
    static CONSTANTS: Lazy<BitGrid10Constants<u16>> = Lazy::new(|| BitGrid10Constants::new());
    BitGrid10::new(&CONSTANTS)
}

//---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::TestHelper;

    #[test]
    fn test() {
        let helper = TestHelper::new(new_bit_grid_10_12);
        helper.basic();
    }
}
