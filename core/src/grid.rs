use std::{fmt, ops, cmp};
use std::collections::HashSet;

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

pub trait CellTrait: Copy + Clone + From<char> {
    fn empty() -> Self;
    fn is_empty(&self) -> bool;
    fn char(&self) -> char;
}

pub trait Grid<C: CellTrait>: Clone {
    fn width(&self) -> X;
    fn height(&self) -> Y;
    fn cell(&self, pos: Vec2) -> C;
    /// NOTE: `cell` may be transformed to another corresponding to internal data structure.
    /// This is the reason why the interface of mutating a cell isn't `cell_mut`.
    fn set_cell(&mut self, pos: Vec2, cell: C);
    fn size(&self) -> Vec2 { Vec2(self.width(), self.height()) }
    fn is_empty(&self) -> bool { self.bottom_padding() == self.height() }
    fn is_inside(&self, pos: Vec2) -> bool {
        0 <= pos.0 && pos.0 < self.width() && 0 <= pos.1 && pos.1 < self.height()
    }
    /// If the all non-empty cells in the `sub` grid were put on this grid, `true` is returned.
    fn put<G: Grid<C>>(&mut self, pos: Vec2, sub: &G) -> bool {
        let mut dirty = false;
        for sub_y in 0..sub.height() {
            for sub_x in 0..sub.width() {
                let sub_pos = (sub_x, sub_y).into();
                let sub_cell = sub.cell(sub_pos);
                if sub_cell.is_empty() {
                    continue;
                }
                let p = pos + sub_pos;
                if !self.is_inside(p) {
                    dirty = true;
                    continue;
                }
                let cell = self.cell(p);
                if !cell.is_empty() {
                    dirty = true;
                }
                self.set_cell(p, sub_cell);
            }
        }
        !dirty
    }
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
    /// Example:
    /// ```
    /// use core::grid::{Grid, CellTrait, SampleGrid};
    ///
    /// let mut grid = SampleGrid::new((3, 3).into());
    /// grid.set_rows_with_strs((1, 1).into(), &[
    ///     "@@",
    ///     "@",
    /// ]);
    ///
    /// assert!(!grid.cell((1, 1).into()).is_empty());
    /// assert!(!grid.cell((1, 2).into()).is_empty());
    /// assert!(!grid.cell((2, 2).into()).is_empty());
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
    fn search_last_pos_where_can_put<G: Grid<C>>(&self, pos: Vec2, sub: &G, direction: Vec2) -> Vec2 {
        let mut p = pos;
        loop {
            let pp = p + direction;
            if !self.can_put(pp, sub) {
                return p;
            }
            p = pp;
        }
    }
    fn is_row_filled(&self, y: Y) -> bool {
        debug_assert!(0 <= y && y < self.height());
        for x in 0..self.width() {
            if !self.cell((x, y).into()).is_empty() {
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
    /// `false` will be returned if any non-empty cells are disposed.
    fn insert_cell_to_rows(&mut self, y: Y, cell: C, n: Y, force: bool) -> bool {
        debug_assert!(self.height() >= y + n);
        let mut are_cells_disposed = false;
        for y in (self.height() - n)..self.height() {
            if !self.is_row_empty(y) {
                if !force {
                    return false;
                }
                are_cells_disposed = true;
            }
            self.fill_row(y, cell);
        }
        for y in (0..(self.height() - n)).rev() {
            self.swap_rows(y, y + n);
        }
        !are_cells_disposed
    }
    fn num_droppable_rows<G: Grid<C>>(&self, pos: Vec2, sub: &G) -> Y {
        if !self.can_put(pos, sub) {
            return 0;
        }
        let mut n: Y = 1;
        while self.can_put((pos.0 - n as Y, pos.1).into(), sub) {
            n += 1;
        }
        n - 1
    }
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
    fn detect_space(&self, pos: Vec2) -> HashSet<Vec2> {
        let mut space = HashSet::new();
        let mut checked = HashSet::new();
        let mut unchecked = HashSet::new();
        unchecked.insert(pos);
        loop {
            let p = match unchecked.iter().next().copied() {
                Some(p) => p,
                None => break,
            };
            let ok = unchecked.remove(&p);
            debug_assert!(ok);
            checked.insert(p);
            if self.cell(p).is_empty() {
                space.insert(p);
            } else {
                continue;
            }
            if p.0 > 0 {
                let mut pp = p.clone();
                pp.0 -= 1;
                if !checked.contains(&pp) {
                    unchecked.insert(pp);
                }
            }
            if p.1 > 0 {
                let mut pp = p.clone();
                pp.1 -= 1;
                if !checked.contains(&pp) {
                    unchecked.insert(pp);
                }
            }
            if p.0 < self.width() - 1 {
                let mut pp = p.clone();
                pp.0 += 1;
                if !checked.contains(&pp) {
                    unchecked.insert(pp);
                }
            }
            if p.1 < self.height() - 1 {
                let mut pp = p.clone();
                pp.1 += 1;
                if !checked.contains(&pp) {
                    unchecked.insert(pp);
                }
            }
        }
        space
    }
    /// Example:
    /// ```
    /// use core::grid::{Grid, SampleGrid};
    /// let mut grid = SampleGrid::new((5, 3).into());
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
    /// use core::grid::{Grid, SampleGrid};
    /// let mut grid = SampleGrid::new((4, 4).into());
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
    fn format(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let c = self.cell((x, y).into()).char();
                write!(f, "{}", c)?;
            }
            if y == 0 {
                break;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}

//--------------------------------------------------------------------------------------------------
// Sample Implementations
//--------------------------------------------------------------------------------------------------

#[derive(Debug, Copy, Clone)]
pub enum SampleCell {
    Empty,
    Block,
}

impl CellTrait for SampleCell {
    fn empty() -> Self { Self::Empty }
    fn is_empty(&self) -> bool { matches!(self, Self::Empty) }
    fn char(&self) -> char {
        match self {
            Self::Block => '@',
            Self::Empty => ' ',
        }
    }
}

impl From<char> for SampleCell {
    fn from(c: char) -> Self {
        match c {
            '@' => Self::Block,
            _ => Self::Empty,
        }
    }
}

#[derive(Clone)]
pub struct SampleGrid {
    size: Vec2,
    cells: Vec<SampleCell>,
}

impl SampleGrid {
    pub fn new(size: Vec2) -> Self {
        Self {
            size,
            cells: vec![SampleCell::empty(); (size.0 * size.1) as usize],
        }
    }
    fn cell_index(&self, pos: Vec2) -> usize {
        (pos.0 + pos.1 * self.size.0) as usize
    }
}

impl Grid<SampleCell> for SampleGrid {
    fn width(&self) -> X { self.size.0 }
    fn height(&self) -> Y { self.size.1 }
    fn cell(&self, pos: Vec2) -> SampleCell {
        *self.cells.get(self.cell_index(pos)).unwrap()
    }
    fn set_cell(&mut self, pos: Vec2, cell: SampleCell) {
        let idx = self.cell_index(pos);
        *self.cells.get_mut(idx).unwrap() = cell;
    }
}

//---

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        //
    }
}
