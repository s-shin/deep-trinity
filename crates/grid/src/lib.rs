pub mod bitgrid;

use std::{fmt, ops, cmp};
use std::collections::HashSet;
use std::marker::PhantomData;
use num_traits::PrimInt;

pub type X = i8;
pub type Y = i8;

#[derive(Debug, Copy, Clone, Default, PartialOrd, PartialEq, Eq, Hash)]
pub struct Vec2(pub X, pub Y);

impl From<(X, Y)> for Vec2 {
    fn from(pos: (X, Y)) -> Self { Self(pos.0, pos.1) }
}

impl ops::Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self { Self(self.0 + other.0, self.1 + other.1) }
}

impl ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self { Self(self.0 - other.0, self.1 - other.1) }
}

impl cmp::Ord for Vec2 {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        (self.0, self.1).cmp(&(other.0, other.1))
    }
}

impl fmt::Display for Vec2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.0, self.1)
    }
}

pub trait Cell: Copy + Clone + From<char> {
    fn empty() -> Self;
    fn any_block() -> Self;
    fn is_empty(&self) -> bool;
    fn is_filled(&self) -> bool { !self.is_empty() }
    fn to_char(&self) -> char {
        if self.is_filled() { '@' } else { ' ' }
    }
    fn from_char(c: char) -> Self {
        match c {
            ' ' => Self::empty(),
            _ => Self::any_block(),
        }
    }
}

pub trait Grid<C: Cell>: Clone {
    fn width(&self) -> X;
    fn height(&self) -> Y;
    /// Returns the cell of the `pos`.
    fn cell(&self, pos: Vec2) -> C;
    // NOTE: Define if required.
    // fn cell_safe(&self, pos: Vec2) -> Result<C>;
    /// Sets the `cell` to the `pos`.
    /// `cell` value might be converted to one bit data (filled or empty).
    fn set_cell(&mut self, pos: Vec2, cell: C);
    //----------------------------------------------------------------------------------------------
    fn size(&self) -> Vec2 { Vec2(self.width(), self.height()) }
    fn is_inside(&self, pos: Vec2) -> bool {
        0 <= pos.0 && pos.0 < self.width() && 0 <= pos.1 && pos.1 < self.height()
    }
    fn put<G: Grid<C>>(&mut self, pos: Vec2, sub: &G) {
        for sub_y in 0..sub.height() {
            for sub_x in 0..sub.width() {
                let sub_pos = (sub_x, sub_y).into();
                let sub_cell = sub.cell(sub_pos);
                if sub_cell.is_empty() {
                    continue;
                }
                let p = pos + sub_pos;
                if !self.is_inside(p) {
                    // dirty
                    continue;
                }
                let cell = self.cell(p);
                if !cell.is_empty() {
                    // dirty
                }
                self.set_cell(p, sub_cell);
            }
        }
    }
    /// Returns false when any filled cells of the `sub` grid are set outside the `self` grid or
    /// are set to the positions of the filled cells of the `self` grid.
    fn can_put<G: Grid<C>>(&self, pos: Vec2, sub: &G) -> bool {
        for sub_y in 0..sub.height() {
            for sub_x in 0..sub.width() {
                let sub_pos = (sub_x, sub_y).into();
                if sub.cell(sub_pos).is_empty() {
                    continue;
                }
                let p = pos + sub_pos;
                if !self.is_inside(p) {
                    return false;
                }
                if !self.cell(p).is_empty() {
                    return false;
                }
            }
        }
        true
    }
    fn fill_row(&mut self, y: Y, cell: C) {
        debug_assert!(0 <= y && y < self.height());
        for x in 0..self.width() {
            self.set_cell((x, y).into(), cell);
        }
    }
    fn fill_all(&mut self, cell: C) {
        for y in 0..self.height() {
            self.fill_row(y, cell);
        }
    }
    fn fill_top(&mut self, n: Y, cell: C) {
        if n <= 0 {
            return;
        }
        let h = self.height();
        if n >= h {
            self.fill_all(cell);
        }
        for y in (h - n)..h {
            self.fill_row(y, cell);
        }
    }
    fn fill_bottom(&mut self, n: Y, cell: C) {
        if n <= 0 {
            return;
        }
        let h = self.height();
        if n >= h {
            self.fill_all(cell);
        }
        for y in 0..n {
            self.fill_row(y, cell);
        }
    }
    /// Example:
    /// ```
    /// use grid::{Grid, Cell, BasicGrid, BinaryCell};
    ///
    /// let mut grid = BasicGrid::<BinaryCell>::new((3, 3).into());
    /// grid.set_rows_with_strs((1, 1).into(), &[
    ///     "@@",
    ///     "@",
    /// ]);
    ///
    /// assert!(grid.cell((1, 1).into()).is_filled());
    /// assert!(grid.cell((1, 2).into()).is_filled());
    /// assert!(grid.cell((2, 2).into()).is_filled());
    /// ```
    fn set_rows_with_strs(&mut self, pos: Vec2, rows: &[&str]) {
        for (dy, row) in rows.iter().rev().enumerate() {
            let y = pos.1 + dy as Y;
            if y < 0 || y >= self.height() {
                return;
            }
            for (dx, c) in row.chars().enumerate() {
                let x = pos.0 + dx as X;
                if x < 0 || x >= self.width() {
                    break;
                }
                self.set_cell((x, y).into(), c.into());
            }
        }
    }
    /// Example:
    /// ```
    /// use grid::{Grid, Cell, BasicGrid, BinaryCell};
    ///
    /// let mut grid = BasicGrid::<BinaryCell>::new((3, 3).into());
    /// grid.set_rows_with_bits((1, 1).into(), 3, 0b011001);
    ///
    /// assert!(grid.cell((1, 1).into()).is_filled());
    /// assert!(grid.cell((1, 2).into()).is_filled());
    /// assert!(grid.cell((2, 2).into()).is_filled());
    /// ```
    fn set_rows_with_bits<I: PrimInt>(&mut self, pos: Vec2, stride: u32, bits: I) {
        set_rows_with_bits(self, pos, stride, bits);
    }
    fn reachable_pos<G: Grid<C>>(&self, mut pos: Vec2, sub: &G, direction: Vec2) -> Vec2 {
        loop {
            let p = pos + direction;
            if !self.can_put(p, sub) {
                return pos;
            }
            pos = p;
        }
    }
    fn is_row_filled(&self, y: Y) -> bool {
        debug_assert!(0 <= y && y < self.height());
        for x in 0..self.width() {
            if self.cell((x, y).into()).is_empty() {
                return false;
            }
        }
        true
    }
    fn is_row_empty(&self, y: Y) -> bool {
        debug_assert!(0 <= y && y < self.height());
        for x in 0..self.width() {
            if !self.cell((x, y).into()).is_empty() {
                return false;
            }
        }
        true
    }
    fn is_col_filled(&self, x: X) -> bool {
        debug_assert!(0 <= x && x < self.width());
        for y in 0..self.height() {
            if self.cell((x, y).into()).is_empty() {
                return false;
            }
        }
        true
    }
    fn is_col_empty(&self, x: X) -> bool {
        debug_assert!(0 <= x && x < self.width());
        for y in 0..self.height() {
            if !self.cell((x, y).into()).is_empty() {
                return false;
            }
        }
        true
    }
    fn bottom_padding(&self) -> Y {
        let mut n = 0;
        for y in 0..self.height() {
            if !self.is_row_empty(y) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn top_padding(&self) -> Y {
        let mut n = 0;
        for y in (0..self.height()).rev() {
            if !self.is_row_empty(y) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn left_padding(&self) -> X {
        let mut n = 0;
        for x in 0..self.width() {
            if !self.is_col_empty(x) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn right_padding(&self) -> X {
        let mut n = 0;
        for x in (0..self.width()).rev() {
            if !self.is_col_empty(x) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn is_empty(&self) -> bool { self.bottom_padding() == self.height() }
    fn num_blocks_of_row(&self, y: Y) -> usize {
        if self.is_row_empty(y) {
            return 0;
        }
        let mut n = 0;
        for x in 0..self.width() {
            if !self.cell((x, y).into()).is_empty() {
                n += 1;
            }
        }
        n
    }
    fn num_blocks(&self) -> usize {
        let mut n = 0;
        for y in 0..self.height() {
            n += self.num_blocks_of_row(y);
        }
        n
    }
    fn swap_rows(&mut self, y1: Y, y2: Y) {
        debug_assert!(0 <= y1 && y1 < self.height());
        debug_assert!(0 <= y2 && y2 < self.height());
        if y1 == y2 {
            return;
        }
        for x in 0..self.width() {
            let c1 = self.cell((x, y1).into());
            let c2 = self.cell((x, y2).into());
            self.set_cell((x, y1).into(), c2);
            self.set_cell((x, y2).into(), c1);
        }
    }
    fn num_filled_rows(&self) -> Y {
        let mut n = 0;
        for y in 0..self.height() {
            if self.is_row_filled(y) {
                n += 1;
            }
        }
        return n;
    }
    fn drop_filled_rows(&mut self) -> Y {
        let mut n = 0;
        for y in 0..self.height() {
            if self.is_row_filled(y) {
                self.fill_row(y, C::empty());
                n += 1
            } else if n > 0 {
                self.swap_rows(y - n, y);
            }
        }
        n
    }
    /// `false` will be returned if any filled cells are cleared.
    fn insert_rows(&mut self, y: Y, cell: C, n: Y) -> bool {
        debug_assert!(self.height() >= y + n);
        let mut cleared = false;
        for y in (self.height() - n)..self.height() {
            if !self.is_row_empty(y) {
                cleared = true;
            }
            self.fill_row(y, cell);
        }
        for y in (0..(self.height() - n)).rev() {
            self.swap_rows(y, y + n);
        }
        !cleared
    }
    fn num_droppable_rows<G: Grid<C>>(&self, pos: Vec2, sub: &G) -> Y {
        let mut n = 0;
        while self.can_put((pos.0, pos.1 - n).into(), sub) {
            n += 1;
        }
        (n - 1).max(0)
    }
    fn traverse(&self, start_pos: Vec2, mut cb: impl FnMut(Vec2, C) -> bool) {
        let mut open = HashSet::new();
        let mut closed = HashSet::new();
        open.insert(start_pos);
        while let Some(p) = open.iter().next().copied() {
            let is_removed = open.remove(&p);
            debug_assert!(is_removed);
            closed.insert(p);
            let cell = self.cell(p);
            if !cb(p, cell) {
                continue;
            }
            if p.0 > 0 {
                let mut pp = p.clone();
                pp.0 -= 1;
                if !closed.contains(&pp) {
                    open.insert(pp);
                }
            }
            if p.1 > 0 {
                let mut pp = p.clone();
                pp.1 -= 1;
                if !closed.contains(&pp) {
                    open.insert(pp);
                }
            }
            if p.0 < self.width() - 1 {
                let mut pp = p.clone();
                pp.0 += 1;
                if !closed.contains(&pp) {
                    open.insert(pp);
                }
            }
            if p.1 < self.height() - 1 {
                let mut pp = p.clone();
                pp.1 += 1;
                if !closed.contains(&pp) {
                    open.insert(pp);
                }
            }
        }
    }
    /// Search a space (empty area) including the `pos`.
    fn search_space(&self, pos: Vec2) -> HashSet<Vec2> {
        let mut space = HashSet::new();
        self.traverse(pos, |p, c| {
            if c.is_empty() {
                space.insert(p);
                return true;
            }
            false
        });
        space
    }
    /// Search spaces in the rect represented by `pos` and `size` of interest.
    fn search_spaces(&self, pos: Vec2, size: Vec2) -> Vec<HashSet<Vec2>> {
        let mut spaces = Vec::new();
        let mut check_list = HashSet::<Vec2>::with_capacity(size.0 as usize * size.1 as usize);
        for y in pos.1..(pos.1 + size.1) {
            for x in pos.0..(pos.0 + size.0) {
                check_list.insert((x, y).into());
            }
        }
        while let Some(start) = check_list.iter().next().copied() {
            if self.cell(start).is_filled() {
                check_list.remove(&start);
                continue;
            }
            let mut space = HashSet::new();
            self.traverse(start, |p, c| {
                if !check_list.remove(&p) {
                    return false;
                }
                if c.is_empty() {
                    space.insert(p);
                    return true;
                }
                false
            });
            spaces.push(space);
        }
        spaces
    }
    /// Example:
    /// ```
    /// use grid::{Grid, BasicGrid, BinaryCell};
    /// let mut grid = BasicGrid::<BinaryCell>::new((5, 3).into());
    /// grid.set_rows_with_strs((0, 0).into(), &[
    ///     "@ @ @",
    ///     "@@ @ ", // -> 2
    ///     "  @@ ", // -> 3
    /// ]);
    /// assert_eq!(5, grid.num_covered_empty_cells());
    /// ```
    fn num_covered_empty_cells(&self) -> usize {
        let mut n = 0;
        let mut xs = HashSet::new();
        for y in (0..self.height()).rev() {
            if self.is_row_empty(y) {
                if xs.is_empty() {
                    continue;
                }
                n += xs.len();
            } else {
                for x in 0..self.width() {
                    if self.cell((x, y).into()).is_empty() {
                        if xs.contains(&x) {
                            n += 1;
                        }
                    } else {
                        xs.insert(x);
                    }
                }
            }
        }
        n
    }
    /// Example:
    /// ```
    /// use grid::{Grid, BasicGrid, BinaryCell};
    /// let mut grid = BasicGrid::<BinaryCell>::new((4, 4).into());
    /// grid.set_rows_with_strs((0, 0).into(), &[
    ///     "@   ",
    ///     "@@ @",
    ///     "@  @",
    ///     "@@@ ",
    /// ]);
    /// assert_eq!(vec![3, 2, 0, 2], grid.contour());
    /// ```
    fn contour(&self) -> Vec<Y> {
        let mut xs = vec![0; self.width() as usize];
        for y in 0..self.height() {
            if self.is_row_empty(y) {
                continue;
            }
            for x in 0..self.width() {
                if !self.cell((x, y).into()).is_empty() {
                    xs[x as usize] = y;
                }
            }
        }
        xs
    }
    fn density(&self) -> f32 {
        self.num_blocks() as f32 / (self.width() * self.height()) as f32
    }
    fn density_without_top_padding(&self) -> f32 {
        self.num_blocks() as f32 / (self.width() * (self.height() - self.top_padding())) as f32
    }
    fn format<Writer: std::fmt::Write>(&self, w: &mut Writer) -> fmt::Result {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let c = self.cell((x, y).into()).to_char();
                write!(w, "{}", c)?;
            }
            if y == 0 {
                break;
            }
            write!(w, "\n")?;
        }
        Ok(())
    }
    fn to_string(&self) -> String {
        let mut s = String::new();
        self.format(&mut s).unwrap();
        s
    }
}

fn set_rows_with_bits<C: Cell, G: Grid<C>, I: PrimInt>(grid: &mut G, pos: Vec2, stride: u32, mut bits: I) {
    while !bits.is_zero() {
        let n = bits.trailing_zeros();
        let x = pos.0 + (n % stride) as X;
        let y = pos.1 + (n / stride) as Y;
        if y < 0 || y >= grid.height() {
            return;
        }
        if x < 0 || x >= grid.width() {
            continue;
        }
        grid.set_cell((x, y).into(), C::any_block());
        bits = bits & (bits - I::one());
    }
}

//--------------------------------------------------------------------------------------------------
// BinaryCell
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub struct BinaryCell(bool);

impl Cell for BinaryCell {
    fn empty() -> Self { Self(false) }
    fn any_block() -> Self { Self(true) }
    fn is_empty(&self) -> bool { !self.0 }
}

impl From<char> for BinaryCell {
    fn from(c: char) -> Self { Self::from_char(c) }
}

//--------------------------------------------------------------------------------------------------
// BasicGrid
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BasicGrid<C: Cell> {
    size: Vec2,
    cells: Vec<C>,
}

impl<C: Cell> BasicGrid<C> {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            cells: vec![C::empty(); size.0 as usize * size.1 as usize],
        }
    }
    fn cell_index(&self, pos: Vec2) -> usize {
        debug_assert!(self.is_inside(pos));
        pos.0 as usize + pos.1 as usize * self.size.0 as usize
    }
    pub fn rotate_cw(&self) -> Self {
        let mut g = Self::new((self.height(), self.width()).into());
        for y in 0..self.height() {
            for x in 0..self.width() {
                g.set_cell((y, self.width() - 1 - x).into(), self.cell((x, y).into()));
            }
        }
        g
    }
}

impl<C: Cell> Grid<C> for BasicGrid<C> {
    fn width(&self) -> X { self.size.0 }
    fn height(&self) -> Y { self.size.1 }
    fn cell(&self, pos: Vec2) -> C {
        *self.cells.get(self.cell_index(pos)).unwrap()
    }
    fn set_cell(&mut self, pos: Vec2, cell: C) {
        let idx = self.cell_index(pos);
        *self.cells.get_mut(idx).unwrap() = cell;
    }
}

impl<C: Cell> fmt::Display for BasicGrid<C> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

//--------------------------------------------------------------------------------------------------
// TestSuite
//--------------------------------------------------------------------------------------------------

pub struct TestSuite<C: Cell, G: Grid<C>, F: Fn() -> G> {
    new_empty_grid: F,
    phantom: PhantomData<fn() -> C>,
}

impl<C: Cell, G: Grid<C>, F: Fn() -> G> TestSuite<C, G, F> {
    pub fn new(new_empty_grid: F) -> Self {
        let g = new_empty_grid();
        assert!(g.is_empty());
        assert!(g.size() >= (5, 5).into());
        Self { new_empty_grid, phantom: PhantomData }
    }
    fn new_empty_grid(&self) -> G { (self.new_empty_grid)() }
    pub fn basic(&self) {
        let mut g = self.new_empty_grid();
        assert!(g.is_empty());

        g.set_cell((1, 1).into(), C::any_block());
        assert!(!g.is_empty());
        assert!(g.cell((1, 1).into()).is_filled());
        assert_eq!(1, g.num_blocks());
        assert!(!g.is_row_empty(1));
        assert!(!g.is_row_filled(1));
        assert!(!g.is_col_empty(1));
        assert!(!g.is_col_filled(1));
        assert_eq!(g.height() - 2, g.top_padding());
        assert_eq!(1, g.bottom_padding());
        assert_eq!(1, g.left_padding());
        assert_eq!(g.width() - 2, g.right_padding());

        g.fill_row(4, C::any_block());
        assert_eq!(g.width() as usize + 1, g.num_blocks());
        assert_eq!(g.width() as usize, g.num_blocks_of_row(4));
        assert!(g.is_row_filled(4));

        g.swap_rows(1, 4);
        assert_eq!(g.width() as usize, g.num_blocks_of_row(1));
        assert_eq!(1, g.num_blocks_of_row(4));

        g.drop_filled_rows();
        assert_eq!(1, g.num_blocks());
        assert_eq!(1, g.num_blocks_of_row(3));

        g.fill_all(C::any_block());
        assert_eq!((g.width() * g.height()) as usize, g.num_blocks());

        g.fill_all(C::empty());
        assert!(g.is_empty());
    }
    pub fn search_space(&self) {
        let mut g = self.new_empty_grid();
        g.set_rows_with_strs((0, 0).into(), &[
            "@@@@ ",
            "   @ ",
            " @  @",
            "  @ @",
            " @  @",
        ]);
        let spaces = g.search_space((0, 0).into());
        let expected = [
            (0, 0), (2, 0), (3, 0),
            (0, 1), (1, 1), (3, 1),
            (0, 2), (2, 2), (3, 2),
            (0, 3), (1, 3), (2, 3),
        ].map(|(x, y)| Vec2(x, y));
        assert_eq!(expected.len(), spaces.len());
        for pos in expected.iter() {
            assert!(spaces.contains(pos));
        }
    }
    pub fn search_spaces(&self) {
        let mut g = self.new_empty_grid();
        g.set_rows_with_strs((0, 0).into(), &[
            "@@@@ ",
            "   @ ",
            " @  @",
            "  @ @",
            " @  @",
        ]);
        let spaces = g.search_spaces((1, 1).into(), (4, 4).into());
        let expected_spaces: Vec<Vec<Vec2>> = vec![
            [(1, 1)].iter().map(|&p| p.into()).collect::<Vec<_>>(),
            [(1, 3), (2, 2), (2, 3), (3, 1), (3, 2)].iter().map(|&p| p.into()).collect::<Vec<_>>(),
            [(4, 3), (4, 4)].iter().map(|&p| p.into()).collect::<Vec<_>>(),
        ];
        assert_eq!(expected_spaces.len(), spaces.len());
        for space in &spaces {
            let expected_space = expected_spaces.iter().find(|s| space.contains(s.first().unwrap()));
            assert!(expected_space.is_some());
            for pos in expected_space.unwrap() {
                assert!(space.contains(pos));
            }
        }
    }
}

//---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite() {
        let suite = TestSuite::new(|| BasicGrid::<BinaryCell>::new((5, 5).into()));
        suite.basic();
        suite.search_space();
        suite.search_spaces();
    }
}
