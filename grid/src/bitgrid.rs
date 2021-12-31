use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::fmt;
use num_traits::PrimInt;
use crate::{Grid, BinaryCell, CellTrait, Vec2, X, Y};

/// This struct contains many constant values to be used by [PrimBitGrid].
#[derive(Clone, Debug)]
pub struct PrimBitGridConstants<Int: PrimInt> {
    pub num_bits: u32,
    pub stride: X,
    pub width: X,
    pub max_height: Y,
    pub height: Y,
    pub cells_mask: Int,
    row_masks: Vec<Int>,
    col_masks: Vec<Int>,
    left_side_cols_masks: Vec<Int>,
    right_side_cols_masks: Vec<Int>,
    bottom_side_rows_masks: Vec<Int>,
    top_side_rows_masks: Vec<Int>,
}

impl<Int: PrimInt> PrimBitGridConstants<Int> {
    pub fn new(width: X, height: Option<Y>, stride: Option<X>) -> Self {
        let num_bits = Int::zero().count_zeros();
        let stride = stride.unwrap_or(width);
        assert!(width <= stride);
        assert!(1 <= stride && stride as u32 <= num_bits);
        let max_height = (num_bits / stride as u32) as Y;
        let height = height.unwrap_or(max_height);
        assert!(1 <= height && height <= max_height);
        let row_masks = {
            let m = (!Int::zero()).unsigned_shr(num_bits - width as u32);
            (0..height).map(|y| m << (y as usize * stride as usize)).collect::<Vec<_>>()
        };
        let col_masks = {
            let mut m = Int::zero();
            for y in 0..height {
                m = m | Int::one() << (y as usize * stride as usize);
            }
            (0..width).map(|x| m << x as usize).collect::<Vec<_>>()
        };
        let cells_mask = {
            let mut m = Int::zero();
            for y in 0..height {
                m = m | row_masks[y as usize];
            }
            m
        };
        let left_side_cols_masks = {
            let w = width as usize;
            let mut r = Vec::with_capacity(w);
            let mut t = Int::zero();
            for x in 0..(w - 1) {
                t = t | col_masks[x];
                r.push(t);
            }
            r
        };
        let right_side_cols_masks = {
            let w = width as usize;
            let mut r = Vec::with_capacity(w);
            let mut t = Int::zero();
            for x in (1..w).rev() {
                t = t | col_masks[x];
                r.push(t);
            }
            r
        };
        let bottom_side_rows_masks = {
            let h = height as usize;
            let mut r = Vec::with_capacity(h);
            let mut t = Int::zero();
            for y in 0..(h - 1) {
                t = t | row_masks[y];
                r.push(t);
            }
            r
        };
        let top_side_rows_masks = {
            let h = height as usize;
            let mut r = Vec::with_capacity(h);
            let mut t = Int::zero();
            for y in (1..h).rev() {
                t = t | row_masks[y];
                r.push(t);
            }
            r
        };
        Self {
            num_bits,
            stride,
            width,
            max_height,
            height,
            cells_mask,
            row_masks,
            col_masks,
            left_side_cols_masks,
            right_side_cols_masks,
            bottom_side_rows_masks,
            top_side_rows_masks,
        }
    }
    pub fn row_mask(&self, y: Y) -> Int {
        debug_assert!(0 <= y && y < self.height);
        self.row_masks[y as usize]
    }
    pub fn col_mask(&self, x: X) -> Int {
        debug_assert!(0 <= x && x < self.width);
        self.col_masks[x as usize]
    }
    pub fn left_side_cols_mask(&self, num_blocks: X) -> Int {
        let n = num_blocks;
        debug_assert!(0 < n);
        self.left_side_cols_masks.get((n - 1) as usize).copied().unwrap_or(self.cells_mask)
    }
    pub fn right_side_cols_mask(&self, num_blocks: X) -> Int {
        let n = num_blocks;
        debug_assert!(0 < n);
        self.right_side_cols_masks.get((n - 1) as usize).copied().unwrap_or(self.cells_mask)
    }
    pub fn left_side_empty_cols_mask(&self, num_empty_cells: X) -> Int {
        let n = num_empty_cells;
        debug_assert!(0 < n);
        self.right_side_cols_masks.get(((self.width - 1) - n) as usize).copied().unwrap_or(Int::zero())
    }
    pub fn right_side_empty_cols_mask(&self, num_empty_cells: X) -> Int {
        let n = num_empty_cells;
        debug_assert!(0 < n);
        self.left_side_cols_masks.get(((self.width - 1) - n) as usize).copied().unwrap_or(Int::zero())
    }
    pub fn top_side_rows_mask(&self, num_blocks: Y) -> Int {
        let n = num_blocks;
        debug_assert!(0 < n);
        self.top_side_rows_masks.get((n - 1) as usize).copied().unwrap_or(self.cells_mask)
    }
    pub fn bottom_side_rows_mask(&self, num_blocks: Y) -> Int {
        let n = num_blocks;
        debug_assert!(0 < n);
        self.bottom_side_rows_masks.get((n - 1) as usize).copied().unwrap_or(self.cells_mask)
    }
    pub fn top_side_empty_rows_mask(&self, num_empty_cells: Y) -> Int {
        let n = num_empty_cells;
        debug_assert!(0 < n);
        self.bottom_side_rows_masks.get(((self.height - 1) - n) as usize).copied().unwrap_or(Int::zero())
    }
    pub fn bottom_side_empty_rows_mask(&self, num_empty_cells: Y) -> Int {
        let n = num_empty_cells;
        debug_assert!(0 < n);
        self.top_side_rows_masks.get(((self.height - 1) - n) as usize).copied().unwrap_or(Int::zero())
    }
}

/// Generally, all [PrimBitGridConstants] instances are global (static) data.
/// This struct helps us generate, store and get these constants.
pub struct PrimBitGridConstantsStore<Int: PrimInt> {
    pub stride: X,
    pub prim_max_height: Y,
    constants_map: HashMap<Vec2, PrimBitGridConstants<Int>>,
}

impl<Int: PrimInt> PrimBitGridConstantsStore<Int> {
    pub fn new(stride: X) -> Self {
        let prim_num_bits = Int::zero().count_zeros();
        let prim_max_height = prim_num_bits as Y / stride;
        Self { stride, prim_max_height, constants_map: HashMap::new() }
    }
    pub fn prepare(&mut self, size: Vec2) {
        if !self.constants_map.contains_key(&size) {
            self.constants_map.insert(size, PrimBitGridConstants::new(size.0, Some(size.1), Some(self.stride)));
        }
    }
    pub fn prepare_for_prim_bit_grid(&mut self, size: Vec2) {
        self.prepare(size);
    }
    pub fn prepare_for_bit_grid(&mut self, size: Vec2) {
        if size.1 <= self.prim_max_height {
            self.prepare_for_prim_bit_grid(size);
        } else {
            self.prepare_for_prim_bit_grid((size.0, self.prim_max_height).into());
            let edge_height = size.1 % self.prim_max_height;
            if edge_height > 0 {
                self.prepare_for_prim_bit_grid((size.0, edge_height).into())
            }
        }
    }
    pub fn get(&self, size: Vec2) -> Option<&PrimBitGridConstants<Int>> { self.constants_map.get(&size) }
}

//---

pub trait BitGridTrait<'a, Int: PrimInt, C: CellTrait>: Grid<C> {
    fn with_store(store: &'a PrimBitGridConstantsStore<Int>, size: Vec2) -> Option<Self>;
    fn put_prim_bit_grid(&mut self, pos: Vec2, other: &PrimBitGrid<Int, C>) {
        self.put(pos, other);
    }
    fn can_put_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> bool {
        self.can_put(pos, other)
    }
    fn num_droppable_rows_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> Y {
        self.num_droppable_rows(pos, other)
    }
    fn reachable_pos_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>, direction: Vec2) -> Vec2 {
        self.reachable_pos(pos, other, direction)
    }
}

//---

/// A [BitGridTrait] implementation by single primitive integer.
#[derive(Clone, Debug)]
pub struct PrimBitGrid<'a, Int: PrimInt, C: CellTrait = BinaryCell> {
    constants: &'a PrimBitGridConstants<Int>,
    cells: Int,
    phantom: PhantomData<fn() -> C>,
}

impl<'a, Int: PrimInt, C: CellTrait> PrimBitGrid<'a, Int, C> {
    pub fn new(constants: &'a PrimBitGridConstants<Int>) -> Self {
        Self { constants, cells: Int::zero(), phantom: PhantomData }
    }
    pub fn constants(&self) -> &'a PrimBitGridConstants<Int> { self.constants }
    fn bit_index(&self, pos: Vec2) -> i32 { self.constants.stride as i32 * pos.1 as i32 + pos.0 as i32 }
    fn cell_mask(&self, pos: Vec2) -> Int {
        let i = self.bit_index(pos);
        assert!(i >= 0);
        Int::one() << i as usize
    }
    fn put_same_stride<OtherCell: CellTrait>(&mut self, pos: Vec2, other: &PrimBitGrid<Int, OtherCell>) {
        assert_eq!(self.constants.stride, other.constants.stride);
        let other_cells = other.cells;
        // Clear left and right side bits before bit shift.
        let other_cells =
            if pos.0 == 0 {
                other_cells
            } else if pos.0 < 0 {
                other_cells & self.constants.left_side_empty_cols_mask(-pos.0)
            } else {
                let n = pos.0 + other.width() - self.width();
                if n > 0 {
                    other_cells & self.constants.right_side_empty_cols_mask(n)
                } else {
                    other_cells
                }
            };
        let i = self.bit_index(pos);
        self.cells = self.cells | if i == 0 {
            other_cells
        } else if i > 0 {
            other_cells << i as usize & self.constants.cells_mask
        } else {
            other_cells.unsigned_shr(-i as u32) & self.constants.cells_mask
        };
    }
    fn can_put_same_stride<OtherCell: CellTrait>(&self, pos: Vec2, other: &PrimBitGrid<Int, OtherCell>) -> bool {
        assert_eq!(self.constants.stride, other.constants.stride);
        if other.is_empty() {
            return true;
        }
        if pos.0 < 0 {
            // Check overflow of the left side of other.
            if other.cells & other.constants.left_side_cols_mask(-pos.0) != Int::zero() {
                return false;
            }
        }
        let n = pos.0 + other.width() - self.width();
        if n > 0 {
            // Check overflow of the right side of other.
            if other.cells & other.constants.right_side_cols_mask(n) != Int::zero() {
                return false;
            }
        }
        if pos.1 < 0 {
            // Check overflow of the bottom side of other.
            if other.cells & other.constants.bottom_side_rows_mask(-pos.1) != Int::zero() {
                return false;
            }
        }
        let n = pos.1 + other.height() - self.height();
        if n > 0 {
            // Check overflow of the top side of other.
            if other.cells & other.constants.top_side_rows_mask(n) != Int::zero() {
                return false;
            }
        }
        let i = self.bit_index(pos);
        self.cells & if i == 0 {
            other.cells
        } else if i > 0 {
            other.cells << i as usize
        } else {
            other.cells.unsigned_shr(-i as u32)
        } == Int::zero()
    }
    fn num_droppable_rows_same_stride(&self, pos: Vec2, sub: &PrimBitGrid<Int, C>) -> Y {
        let mut n = 0;
        while self.can_put_same_stride((pos.0, pos.1 - n).into(), sub) {
            n += 1;
        }
        n
    }
    pub fn swap_row_with_other<OtherCell: CellTrait>(&mut self, y: Y, other: &mut PrimBitGrid<Int, OtherCell>, other_y: Y) {
        assert_eq!(self.constants.stride, other.constants.stride);
        assert_eq!(self.constants.width, other.constants.width);
        assert!(0 <= y && y < self.constants.height);
        assert!(0 <= other_y && other_y < other.constants.height);
        let stride = self.constants.stride as u32;
        let mask = self.constants.row_mask(y);
        let row = self.cells & mask;
        let other_mask = other.constants.row_mask(other_y);
        let other_row = other.cells & other_mask;
        self.cells = (self.cells & !mask) | if y == other_y {
            other_row
        } else if y > other_y {
            other_row << ((y - other_y) as usize * stride as usize)
        } else {
            other_row.unsigned_shr((other_y - y) as u32 * stride)
        };
        other.cells = (other.cells & !other_mask) | if y == other_y {
            row
        } else if other_y > y {
            row << ((other_y - y) as usize * stride as usize)
        } else {
            row.unsigned_shr((y - other_y) as u32 * stride)
        };
    }
    fn reachable_pos_same_stride(&self, mut pos: Vec2, other: &PrimBitGrid<Int, C>, direction: Vec2) -> Vec2 {
        loop {
            let p = pos + direction;
            if !self.can_put_same_stride(p, other) {
                return pos;
            }
            pos = p;
        }
    }
}

impl<'a, Int: PrimInt, C: CellTrait> BitGridTrait<'a, Int, C> for PrimBitGrid<'a, Int, C> {
    fn with_store(store: &'a PrimBitGridConstantsStore<Int>, size: Vec2) -> Option<Self> {
        store.get(size).map(|c| Self::new(c))
    }
    fn put_prim_bit_grid(&mut self, pos: Vec2, other: &PrimBitGrid<Int, C>) {
        if self.constants.stride == other.constants.stride {
            return self.put_same_stride(pos, other);
        }
        self.put(pos, other);
    }
    fn can_put_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> bool {
        if self.constants.stride == other.constants.stride {
            return self.can_put_same_stride(pos, &other);
        }
        self.can_put(pos, other)
    }
    fn num_droppable_rows_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> Y {
        if self.constants.stride == other.constants.stride {
            return self.num_droppable_rows_same_stride(pos, &other);
        }
        self.num_droppable_rows(pos, other)
    }
    fn reachable_pos_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>, direction: Vec2) -> Vec2 {
        if self.constants.stride == other.constants.stride {
            return self.reachable_pos_same_stride(pos, other, direction);
        }
        self.reachable_pos(pos, other, direction)
    }
}

impl<'a, Int: PrimInt, C: CellTrait> Grid<C> for PrimBitGrid<'a, Int, C> {
    fn width(&self) -> X { self.constants.width }
    fn height(&self) -> Y { self.constants.height }
    fn cell(&self, pos: Vec2) -> C {
        if self.cells & self.cell_mask(pos) == Int::zero() { C::empty() } else { C::any_block() }
    }
    fn set_cell(&mut self, pos: Vec2, cell: C) {
        let m = self.cell_mask(pos);
        if cell.is_empty() {
            self.cells = self.cells & !m;
        } else {
            self.cells = self.cells | m;
        }
        debug_assert!(self.cells & !self.constants.cells_mask == Int::zero());
    }
    fn is_empty(&self) -> bool { self.cells == Int::zero() }
    fn fill_row(&mut self, y: Y, cell: C) {
        let m = self.constants.row_mask(y);
        if cell.is_empty() {
            self.cells = self.cells & !m;
        } else {
            self.cells = self.cells | m;
        }
    }
    fn fill_all(&mut self, cell: C) {
        self.cells = if cell.is_empty() { Int::zero() } else { self.constants.cells_mask };
    }
    fn fill_top(&mut self, n: Y, cell: C) {
        if n <= 0 {
            return;
        }
        if n >= self.constants.height {
            return self.fill_all(cell);
        }
        self.cells = if cell.is_empty() {
            self.cells & self.constants.top_side_empty_rows_mask(n)
        } else {
            self.cells & self.constants.top_side_rows_mask(n)
        };
    }
    fn fill_bottom(&mut self, n: Y, cell: C) {
        if n <= 0 {
            return;
        }
        if n >= self.constants.height {
            return self.fill_all(cell);
        }
        self.cells = if cell.is_empty() {
            self.cells & self.constants.bottom_side_empty_rows_mask(n)
        } else {
            self.cells & self.constants.bottom_side_rows_mask(n)
        };
    }
    fn is_row_filled(&self, y: Y) -> bool {
        let m = self.constants.row_mask(y);
        self.cells & m == m
    }
    fn is_row_empty(&self, y: Y) -> bool {
        let m = self.constants.row_mask(y);
        self.cells & m == Int::zero()
    }
    fn is_col_filled(&self, x: X) -> bool {
        let m = self.constants.col_mask(x);
        self.cells & m == m
    }
    fn is_col_empty(&self, x: X) -> bool {
        let m = self.constants.col_mask(x);
        self.cells & m == Int::zero()
    }
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
        let m1 = self.constants.row_mask(y1);
        let m2 = self.constants.row_mask(y2);
        self.cells = (self.cells & !m1 & !m2) | (self.cells & m1) << dy_shift | (self.cells & m2) >> dy_shift;
    }
    fn num_blocks_of_row(&self, y: Y) -> usize {
        let m = self.constants.row_mask(y);
        (self.cells & m).count_ones() as usize
    }
    fn num_blocks(&self) -> usize {
        debug_assert!(self.cells & !self.constants.cells_mask == Int::zero());
        self.cells.count_ones() as usize
    }
}

impl<'a, Int: PrimInt, C: CellTrait> fmt::Display for PrimBitGrid<'a, Int, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

impl<'a, Int: PrimInt, C: CellTrait> PartialEq for PrimBitGrid<'a, Int, C> {
    fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.cells == other.cells
    }
}

impl<'a, Int: PrimInt, C: CellTrait> Eq for PrimBitGrid<'a, Int, C> {}

impl<'a, Int: PrimInt + Hash, C: CellTrait> Hash for PrimBitGrid<'a, Int, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size().hash(state);
        self.cells.hash(state);
    }
}

//---

/// A [BitGridTrait] implementation by multiple primitive integers.
#[derive(Clone, Debug)]
pub struct BasicBitGrid<'a, Int: PrimInt, C: CellTrait = BinaryCell> {
    size: Vec2,
    prim_grids: Vec<PrimBitGrid<'a, Int, C>>,
    prim_height: Y,
}

impl<'a, Int: PrimInt, C: CellTrait> BasicBitGrid<'a, Int, C> {
    pub fn new(repeated: &'a PrimBitGridConstants<Int>, n: Y, edge: Option<&'a PrimBitGridConstants<Int>>) -> Self {
        debug_assert!(n >= 0);
        debug_assert!(edge.is_none() || repeated.width == edge.unwrap().width);
        let size = Vec2(repeated.width, repeated.height * n + edge.map_or(0, |c| c.height));
        assert!(0 < size.0);
        assert!(0 < size.1);
        let num_prim_grids = (n + edge.map_or(0, |_| 1)) as usize;
        let mut prim_grids = Vec::with_capacity(num_prim_grids);
        for _ in 0..n {
            prim_grids.push(PrimBitGrid::new(repeated));
        }
        if let Some(c) = edge {
            prim_grids.push(PrimBitGrid::new(c));
        }
        debug_assert_eq!(num_prim_grids, prim_grids.len());
        let prim_height = repeated.height;
        Self { size, prim_grids, prim_height }
    }
    fn first_prim_grid(&self) -> &PrimBitGrid<'a, Int, C> { self.prim_grids.first().unwrap() }
    pub fn first_prim_grid_info(&self, y: Y) -> (usize, Y) {
        // Example #1:
        // 0     1     2     3
        // 0123450123450123450123 (prim_height = 6)
        //          @@@@@@@@@@ (y = 9, height = 10)
        // => i = 1, y = 3
        //
        // Example #2:
        //   0 1 2 3
        //   0101010 (prim_height = 2)
        // @@@@@@@@@@ (y = -2, height = 10)
        // => i = 0, y = -2
        if y >= 0 {
            ((y / self.prim_height) as usize, y % self.prim_height)
        } else {
            (0, y)
        }
    }
    pub fn prim_grid_info(&self, y: Y, height: Y) -> (usize, Y, usize) {
        let (first_i, first_y) = self.first_prim_grid_info(y);
        let (last_i, _) = self.first_prim_grid_info(y + height - 1);
        (first_i, first_y, last_i)
    }
    fn put_same_stride_prim(&mut self, pos: Vec2, other: &PrimBitGrid<Int, C>) {
        let (mut i, mut y, last_i) = self.prim_grid_info(pos.1, other.height());
        while i <= last_i {
            self.prim_grids.get_mut(i).unwrap().put_prim_bit_grid((pos.0, y).into(), other);
            i += 1;
            y -= self.prim_height;
        }
    }
    fn can_put_same_stride_prim(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> bool {
        let (mut i, mut y, last_i) = self.prim_grid_info(pos.1, other.height());
        while i <= last_i {
            let mut g = other.clone();
            if i < last_i {
                // Example:
                // -------------------> Y
                // ...|0123456789|... (height = 10)
                //           __@@@__ (y = 6, height = 7)
                //               <-> 3 rows should be masked.
                let masked_height = y + g.height() - self.prim_height;
                if masked_height > 0 {
                    g.fill_top(masked_height, C::empty());
                }
            }
            if i > 0 {
                // Example:
                // -------------------> Y
                // ...|0123456789|... (height = 10)
                // __@@@__ (y = -3, height = 7)
                // <-> 3 rows should be masked.
                let masked_height = -y;
                if masked_height > 0 {
                    g.fill_bottom(masked_height, C::empty());
                }
            }
            if !self.prim_grids.get(i).unwrap().can_put_prim_bit_grid((pos.0, y).into(), &g) {
                return false;
            }
            i += 1;
            y -= self.prim_height;
        }
        true
    }
    fn num_droppable_rows_same_stride_prim(&self, pos: Vec2, sub: &PrimBitGrid<Int, C>) -> Y {
        assert!(!sub.is_empty());
        let mut n = 0;
        while self.can_put_same_stride_prim((pos.0, pos.1 - n).into(), sub) {
            n += 1;
        }
        (n - 1).max(0)
    }
    fn reachable_pos_same_stride(&self, mut pos: Vec2, other: &PrimBitGrid<Int, C>, direction: Vec2) -> Vec2 {
        loop {
            let p = pos + direction;
            if !self.can_put_same_stride_prim(p, other) {
                return pos;
            }
            pos = p;
        }
    }
}

impl<'a, Int: PrimInt, C: CellTrait> BitGridTrait<'a, Int, C> for BasicBitGrid<'a, Int, C> {
    fn with_store(store: &'a PrimBitGridConstantsStore<Int>, size: Vec2) -> Option<Self> {
        if size.0 <= 0 || size.1 <= 0 {
            return None;
        }
        if size.1 <= store.prim_max_height {
            store.get(size).map(|c| Self::new(c, 1, None))
        } else {
            let c1 = store.get((size.0, store.prim_max_height).into());
            if c1.is_none() {
                return None;
            }
            let edge_height = size.1 % store.prim_max_height;
            let c2 = if edge_height > 0 {
                store.get((size.0, edge_height).into())
            } else {
                None
            };
            Some(Self::new(c1.unwrap(), size.1 / store.prim_max_height, c2))
        }
    }
    fn put_prim_bit_grid(&mut self, pos: Vec2, other: &PrimBitGrid<Int, C>) {
        let g = self.first_prim_grid();
        if g.constants.stride == other.constants.stride {
            return self.put_same_stride_prim(pos, &other);
        }
        self.put(pos, other);
    }
    fn can_put_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> bool {
        if self.first_prim_grid().constants.stride == other.constants.stride {
            return self.can_put_same_stride_prim(pos, &other);
        }
        self.can_put(pos, other)
    }
    fn num_droppable_rows_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>) -> Y {
        if self.first_prim_grid().constants.stride == other.constants.stride {
            return self.num_droppable_rows_same_stride_prim(pos, &other);
        }
        self.num_droppable_rows(pos, other)
    }
    fn reachable_pos_of_prim_bit_grid(&self, pos: Vec2, other: &PrimBitGrid<Int, C>, direction: Vec2) -> Vec2 {
        if self.first_prim_grid().constants.stride == other.constants.stride {
            return self.reachable_pos_same_stride(pos, other, direction);
        }
        self.reachable_pos(pos, other, direction)
    }
}

impl<'a, Int: PrimInt, C: CellTrait> Grid<C> for BasicBitGrid<'a, Int, C> {
    fn width(&self) -> X { self.size.0 }
    fn height(&self) -> Y { self.size.1 }
    fn cell(&self, pos: Vec2) -> C {
        let (i, y) = self.first_prim_grid_info(pos.1);
        self.prim_grids.get(i).unwrap().cell((pos.0, y).into())
    }
    fn set_cell(&mut self, pos: Vec2, cell: C) {
        let (i, y) = self.first_prim_grid_info(pos.1);
        self.prim_grids.get_mut(i).unwrap().set_cell((pos.0, y).into(), cell);
    }
    fn is_empty(&self) -> bool {
        self.prim_grids.iter().find(|g| !g.is_empty()).is_none()
    }
    fn fill_row(&mut self, y: Y, cell: C) {
        let (i, y) = self.first_prim_grid_info(y);
        self.prim_grids.get_mut(i).unwrap().fill_row(y, cell);
    }
    fn fill_all(&mut self, cell: C) {
        for g in self.prim_grids.iter_mut() {
            g.fill_all(cell);
        }
    }
    fn is_row_filled(&self, y: Y) -> bool {
        let (i, y) = self.first_prim_grid_info(y);
        self.prim_grids.get(i).unwrap().is_row_filled(y)
    }
    fn is_row_empty(&self, y: Y) -> bool {
        let (i, y) = self.first_prim_grid_info(y);
        self.prim_grids.get(i).unwrap().is_row_empty(y)
    }
    fn is_col_filled(&self, x: X) -> bool {
        self.prim_grids.iter().find(|g| !g.is_col_filled(x)).is_none()
    }
    fn is_col_empty(&self, x: X) -> bool {
        self.prim_grids.iter().find(|g| !g.is_col_empty(x)).is_none()
    }
    fn swap_rows(&mut self, mut y1: Y, mut y2: Y) {
        if y1 == y2 {
            return;
        }
        if y1 > y2 {
            std::mem::swap(&mut y1, &mut y2);
        }
        let (i1, y1) = self.first_prim_grid_info(y1);
        let (i2, y2) = self.first_prim_grid_info(y2);
        if i1 == i2 {
            self.prim_grids.get_mut(i1).unwrap().swap_rows(y1, y2);
        } else {
            debug_assert!(i1 < i2);
            let (left, right) = self.prim_grids.split_at_mut(i1 + 1);
            let g1 = left.get_mut(i1).unwrap();
            let g2 = right.get_mut(i2 - i1 - 1).unwrap();
            g1.swap_row_with_other(y1, g2, y2);
        }
    }
    fn num_blocks_of_row(&self, y: Y) -> usize {
        let (i, y) = self.first_prim_grid_info(y);
        self.prim_grids.get(i).unwrap().num_blocks_of_row(y)
    }
    fn num_blocks(&self) -> usize {
        self.prim_grids.iter().fold(0, |n, g| n + g.num_blocks())
    }
}

impl<'a, Int: PrimInt, C: CellTrait> fmt::Display for BasicBitGrid<'a, Int, C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

impl<'a, Int: PrimInt, C: CellTrait> PartialEq for BasicBitGrid<'a, Int, C> {
    fn eq(&self, other: &Self) -> bool {
        self.size() == other.size() && self.prim_grids == other.prim_grids
    }
}

impl<'a, Int: PrimInt, C: CellTrait> Eq for BasicBitGrid<'a, Int, C> {}

impl<'a, Int: PrimInt + Hash, C: CellTrait> Hash for BasicBitGrid<'a, Int, C> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size().hash(state);
        self.prim_grids.hash(state);
    }
}

//---

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Vec2, CellTrait, Grid, TestSuite};

    #[test]
    fn test_prim_bit_grid_constants_u32_10_none_none() {
        let c = PrimBitGridConstants::<u32>::new(10, None, None);

        assert_eq!(32, c.num_bits);
        assert_eq!(10, c.width);
        assert_eq!(3, c.max_height);
        assert_eq!(3, c.height);

        assert_eq!(&[
            0b1111111111,
            0b1111111111_0000000000,
            0b1111111111_0000000000_0000000000,
        ], c.row_masks.as_slice());

        assert_eq!(&[
            0b0000000001_0000000001_0000000001,
            0b0000000010_0000000010_0000000010,
            0b0000000100_0000000100_0000000100,
            0b0000001000_0000001000_0000001000,
            0b0000010000_0000010000_0000010000,
            0b0000100000_0000100000_0000100000,
            0b0001000000_0001000000_0001000000,
            0b0010000000_0010000000_0010000000,
            0b0100000000_0100000000_0100000000,
            0b1000000000_1000000000_1000000000,
        ], c.col_masks.as_slice());

        assert_eq!(0b1111111111_1111111111_1111111111, c.cells_mask);

        assert_eq!(&[
            0b0000000001_0000000001_0000000001,
            0b0000000011_0000000011_0000000011,
            0b0000000111_0000000111_0000000111,
            0b0000001111_0000001111_0000001111,
            0b0000011111_0000011111_0000011111,
            0b0000111111_0000111111_0000111111,
            0b0001111111_0001111111_0001111111,
            0b0011111111_0011111111_0011111111,
            0b0111111111_0111111111_0111111111,
        ], c.left_side_cols_masks.as_slice());

        assert_eq!(&[
            0b1000000000_1000000000_1000000000,
            0b1100000000_1100000000_1100000000,
            0b1110000000_1110000000_1110000000,
            0b1111000000_1111000000_1111000000,
            0b1111100000_1111100000_1111100000,
            0b1111110000_1111110000_1111110000,
            0b1111111000_1111111000_1111111000,
            0b1111111100_1111111100_1111111100,
            0b1111111110_1111111110_1111111110,
        ], c.right_side_cols_masks.as_slice());

        assert_eq!(&[
            0b0000000000_0000000000_1111111111,
            0b0000000000_1111111111_1111111111,
        ], c.bottom_side_rows_masks.as_slice());

        assert_eq!(&[
            0b1111111111_0000000000_0000000000,
            0b1111111111_1111111111_0000000000,
        ], c.top_side_rows_masks.as_slice());
    }

    #[test]
    fn test_prim_bit_grid_constants_u32_5_2_10() {
        let c = PrimBitGridConstants::<u32>::new(5, Some(2), Some(10));

        assert_eq!(32, c.num_bits);
        assert_eq!(10, c.stride);
        assert_eq!(5, c.width);
        assert_eq!(3, c.max_height);
        assert_eq!(2, c.height);

        assert_eq!(&[
            0b0000011111,
            0b0000011111_0000000000,
        ], c.row_masks.as_slice());

        assert_eq!(&[
            0b0000000001_0000000001,
            0b0000000010_0000000010,
            0b0000000100_0000000100,
            0b0000001000_0000001000,
            0b0000010000_0000010000,
        ], c.col_masks.as_slice());

        assert_eq!(0b0000011111_0000011111, c.cells_mask);

        assert_eq!(&[
            0b0000000001_0000000001,
            0b0000000011_0000000011,
            0b0000000111_0000000111,
            0b0000001111_0000001111,
        ], c.left_side_cols_masks.as_slice());

        assert_eq!(&[
            0b0000010000_0000010000,
            0b0000011000_0000011000,
            0b0000011100_0000011100,
            0b0000011110_0000011110,
        ], c.right_side_cols_masks.as_slice());

        assert_eq!(&[
            0b0000000000_0000011111,
        ], c.bottom_side_rows_masks.as_slice());

        assert_eq!(&[
            0b0000011111_0000000000,
        ], c.top_side_rows_masks.as_slice());
    }

    #[test]
    fn test_prim_bit_grid_basic() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u64>::new(10);
            r.prepare_for_prim_bit_grid((10, 6).into());
            r
        };
        let helper = TestSuite::new(|| PrimBitGrid::<_, BinaryCell>::with_store(&store, (10, 6).into()).unwrap());
        helper.basic();
    }

    #[test]
    fn test_prim_bit_grid_misc() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u64>::new(10);
            r.prepare_for_bit_grid((10, 40).into());
            r.prepare_for_prim_bit_grid((3, 3).into());
            r
        };
        let mut g = PrimBitGrid::<_, BinaryCell>::with_store(&store, (3, 3).into()).unwrap();
        g.set_rows_with_strs((0, 0).into(), &[
            " @@",
            " @@",
            "   ",
        ]);
        assert_eq!(1, g.bottom_padding());
        let mut pf = BasicBitGrid::<_, BinaryCell>::with_store(&store, (10, 40).into()).unwrap();
        assert!(!pf.can_put_same_stride_prim((8, 0).into(), &g));
        let p = pf.reachable_pos((7, 0).into(), &g, (1, 0).into());
        assert_eq!(Vec2(7, 0), p);
        assert!(pf.can_put_same_stride_prim((0, -1).into(), &g));
        assert!(!pf.can_put_same_stride_prim((0, -2).into(), &g));
        assert_eq!(1, pf.num_droppable_rows_same_stride_prim((0, 0).into(), &g));
        assert_eq!(19, pf.num_droppable_rows_same_stride_prim((3, 18).into(), &g));
        pf.fill_bottom(18, BinaryCell::Block);
        assert!(pf.can_put_prim_bit_grid((3, 18).into(), &g));
    }

    #[test]
    fn test_prim_bit_grid_put_same_stride() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u64>::new(10);
            r.prepare_for_prim_bit_grid((10, 6).into());
            r
        };
        let mut g1 = PrimBitGrid::<_, BinaryCell>::with_store(&store, (10, 6).into()).unwrap();
        let mut g2 = g1.clone();
        g2.set_rows_with_strs((1, 1).into(), &[
            " @@ ",
            "@ @@",
            "@@ @",
            " @@ ",
        ]);
        g1.put_same_stride((-2, -2).into(), &g2);
        assert_eq!(6, g1.num_blocks());
        let mut block_positions: Vec<(i8, i8)> = vec![
            (0, 0), (0, 2), (1, 1), (1, 2), (2, 0), (2, 1),
        ];
        for pos in &block_positions {
            assert!(g1.cell((*pos).into()).is_block());
        }
        g1.put_same_stride((6, 2).into(), &g2);
        assert_eq!(12, g1.num_blocks());
        block_positions.extend([
            (7, 4), (7, 5), (8, 3), (8, 4), (9, 3), (9, 5),
        ].iter());
        for pos in &block_positions {
            assert!(g1.cell((*pos).into()).is_block());
        }
    }

    #[test]
    fn test_prim_bit_grid_can_put_same_stride() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u64>::new(10);
            r.prepare_for_prim_bit_grid((10, 6).into());
            r
        };
        let mut g1 = PrimBitGrid::<_, BinaryCell>::with_store(&store, (10, 6).into()).unwrap();
        let mut g2 = g1.clone();
        g2.set_rows_with_strs((1, 1).into(), &[
            " @@ ",
            "@ @@",
            "@@ @",
            " @@ ",
        ]);
        assert!(g1.can_put_same_stride((-1, -1).into(), &g2));
        assert!(!g1.can_put_same_stride((-2, 0).into(), &g2));
        assert!(!g1.can_put_same_stride((0, -2).into(), &g2));
        g1.put_same_stride((-1, -1).into(), &g2);
        assert!(!g1.can_put_same_stride((-1, -1).into(), &g2));
    }

    #[test]
    fn test_basic_bit_grid_basic() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u32>::new(10);
            r.prepare_for_bit_grid((10, 6).into());
            r
        };
        let helper = TestSuite::new(|| BasicBitGrid::<_>::with_store(&store, (10, 6).into()).unwrap());
        helper.basic();
    }

    #[test]
    fn test_basic_bit_grid_put_same_stride_prim() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u32>::new(10);
            r.prepare_for_prim_bit_grid((10, 3).into());
            r.prepare_for_bit_grid((10, 6).into());
            r
        };
        let mut g1 = BasicBitGrid::<_>::with_store(&store, (10, 6).into()).unwrap();
        let mut g2 = PrimBitGrid::<_>::with_store(&store, (10, 3).into()).unwrap();
        g2.set_rows_with_strs((1, 1).into(), &["@", "@"]);
        g1.put_same_stride_prim((1, 1).into(), &g2);
        assert!(g1.cell((2, 2).into()).is_block());
        assert!(g1.cell((2, 3).into()).is_block());
    }

    #[test]
    fn test_basic_bit_grid_can_put_same_stride_prim() {
        let store = {
            let mut r = PrimBitGridConstantsStore::<u64>::new(10);
            r.prepare_for_bit_grid((10, 20).into());
            r.prepare_for_prim_bit_grid((10, 6).into());
            r
        };
        let mut g1 = BasicBitGrid::<_>::with_store(&store, (10, 20).into()).unwrap();
        let mut g2 = PrimBitGrid::<_>::with_store(&store, (10, 6).into()).unwrap();
        g2.fill_all(BinaryCell::Block);
        for param in [
            ((1, 0), false),
            ((-1, 0), false),
            ((0, -1), false),
            ((0, 0), true),
            ((0, 1), true),
            ((0, 2), true),
            ((0, 3), true),
            ((0, 4), true),
            ((0, 5), true),
            ((0, 6), true),
            ((0, 12), true),
            ((0, 13), true),
            ((0, 14), true),
            ((0, 15), false),
        ].iter() {
            let pos = Vec2::from(param.0);
            assert_eq!(param.1, g1.can_put_same_stride_prim(pos, &g2), "{}", pos);
        }
        g1.set_cell((0, 0).into(), BinaryCell::Block);
        for param in [
            ((0, 0), false),
            ((0, 1), true),
        ].iter() {
            let pos = Vec2::from(param.0);
            assert_eq!(param.1, g1.can_put_same_stride_prim(pos, &g2), "{}", pos);
        }
    }

    #[test]
    fn test_prim_bit_grid_constants_store() {
        {
            let mut c = PrimBitGridConstantsStore::<u64>::new(10);
            c.prepare_for_prim_bit_grid(Vec2(10, 5));
            assert!(c.get(Vec2(10, 4)).is_none());
            assert!(c.get(Vec2(10, 5)).is_some());
        }
        {
            let mut c = PrimBitGridConstantsStore::<u64>::new(10);
            c.prepare_for_bit_grid((10, 10).into());
            assert!(c.get(Vec2(10, 4)).is_some());
            assert!(c.get(Vec2(10, 6)).is_some());
            assert!(c.get(Vec2(10, 8)).is_none());
        }
    }
}
