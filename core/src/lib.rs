use std::fmt;

#[derive(Copy, Clone, Debug)]
enum Rotation {
    Deg0,
    Deg90,
    Deg180,
    Deg270,
}

const ROTATIONS: [Rotation; 4] = [
    Rotation::Deg0, Rotation::Deg90, Rotation::Deg180, Rotation::Deg270,
];

impl From<i8> for Rotation {
    fn from(n: i8) -> Self { ROTATIONS[(n % 4) as usize] }
}

impl Rotation {
    fn rotate(self, n: i8) -> Self {
        ((self as i8 + n) % 4).into()
    }
}

//---

// 0: Empty
// 1: Any
// 2-8: S, Z, L, J, I, T, O
// 9: Garbage
struct CellType(u8);

#[derive(Copy, Clone, Debug)]
enum Piece {
    S,
    Z,
    L,
    J,
    I,
    T,
    O,
}

impl Into<CellType> for Piece {
    fn into(self) -> CellType { CellType(2 + (self as u8)) }
}

#[derive(Copy, Clone, Debug)]
enum Block {
    Any,
    Piece(Piece),
    Garbage,
}

impl Into<CellType> for Block {
    fn into(self) -> CellType {
        match self {
            Block::Any => CellType(1),
            Block::Piece(p) => p.into(),
            Block::Garbage => CellType(9),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Cell {
    Empty,
    Block(Block),
}

impl Into<CellType> for Cell {
    fn into(self) -> CellType {
        match self {
            Cell::Empty => CellType(0),
            Cell::Block(b) => b.into(),
        }
    }
}

const CELL_CHARS: &'static str = " @SZLJITO#";

impl Cell {
    fn is_empty(self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false
        }
    }
    fn char(self) -> char {
        let id: CellType = self.into();
        CELL_CHARS.chars().nth(id.0 as usize).unwrap()
    }
}

//---

type SizeX = u8;
type SizeY = u8;
type Size = (SizeX, SizeY);
type PosX = i8;
type PosY = i8;
type Pos = (PosX, PosY);
type UPosX = SizeX;
type UPosY = SizeY;
type UPos = Size;

trait Grid: fmt::Display {
    fn size(&self) -> Size;
    fn width(&self) -> SizeX { self.size().0 }
    fn height(&self) -> SizeY { self.size().1 }
    fn get_cell(&self, pos: UPos) -> Cell;
    fn set_cell(&mut self, pos: UPos, cell: Cell);
    fn has_cell(&self, pos: UPos) -> bool { !self.get_cell(pos).is_empty() }
    fn put<G: Grid>(&mut self, pos: Pos, other: &G) -> bool {
        let mut dirty = false;
        for other_y in 0..other.height() {
            for other_x in 0..other.width() {
                let other_cell = other.get_cell((other_x, other_y));
                if other_cell.is_empty() {
                    continue;
                }
                let x = pos.0 + other_x as i8;
                let y = pos.1 + other_y as i8;
                if x < 0 || self.width() as i8 <= x || y < 0 || self.height() as i8 <= y {
                    dirty = true;
                    continue;
                }
                let p = (x as UPosX, y as UPosY);
                let cell = self.get_cell(p);
                if !cell.is_empty() {
                    dirty = true;
                }
                self.set_cell(p, other_cell);
            }
        }
        !dirty
    }
    fn can_put<G: Grid>(&self, pos: Pos, other: &G) -> bool {
        for other_y in 0..other.height() {
            for other_x in 0..other.width() {
                if !other.has_cell((other_x, other_y)) {
                    continue;
                }
                let x = pos.0 + other_x as i8;
                let y = pos.1 + other_y as i8;
                if x < 0 || self.width() as i8 <= x || y < 0 || self.height() as i8 <= y {
                    return false;
                }
                let p = (x as UPosX, y as UPosY);
                if self.has_cell(p) {
                    return false;
                }
            }
        }
        true
    }
    fn is_row_filled(&self, y: UPosY) -> bool {
        for x in 0..self.width() {
            if !self.has_cell((x, y)) {
                return false;
            }
        }
        true
    }
    fn is_row_empty(&self, y: UPosY) -> bool {
        for x in 0..self.width() {
            if self.has_cell((x, y)) {
                return false;
            }
        }
        true
    }
    fn is_col_filled(&self, x: UPosX) -> bool {
        for y in 0..self.height() {
            if !self.has_cell((x, y)) {
                return false;
            }
        }
        true
    }
    fn is_col_empty(&self, x: UPosX) -> bool {
        for y in 0..self.height() {
            if self.has_cell((x, y)) {
                return false;
            }
        }
        true
    }
    fn bottom_padding(&self) -> SizeY {
        let mut n = 0;
        for y in 0..self.height() {
            if !self.is_row_empty(y) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn top_padding(&self) -> SizeY {
        let mut n = 0;
        for y in (0..self.height()).rev() {
            if !self.is_row_empty(y) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn left_padding(&self) -> SizeX {
        let mut n = 0;
        for x in 0..self.width() {
            if !self.is_col_empty(x) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn right_padding(&self) -> SizeX {
        let mut n = 0;
        for x in (0..self.width()).rev() {
            if !self.is_col_empty(x) {
                return n;
            }
            n += 1;
        }
        n
    }
    fn swap_rows(&mut self, y1: UPosY, y2: UPosY) {
        debug_assert!(y1 < self.height());
        debug_assert!(y2 < self.height());
        if y1 == y2 {
            return;
        }
        for x in 0..self.width() {
            let c1 = self.get_cell((x, y1));
            let c2 = self.get_cell((x, y2));
            self.set_cell((x, y1), c2);
            self.set_cell((x, y2), c1);
        }
    }
    fn set_row(&mut self, y: UPosY, cell: Cell) {
        debug_assert!(y < self.height());
        for x in 0..self.width() {
            self.set_cell((x, y), cell);
        }
    }
    fn drop_filled_row(&mut self) -> SizeY {
        let mut n = 0;
        for y in 0..self.height() {
            if self.is_row_filled(y) {
                self.set_row(y, Cell::Empty);
                n += 1
            } else if n > 0 {
                self.swap_rows(y - n, y);
            }
        }
        n
    }
    fn format(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let c = self.get_cell((x, y)).char();
                if let Err(e) = write!(f, "{}", c) {
                    return Err(e);
                }
            }
            if y == 0 {
                break;
            }
            if let Err(e) = write!(f, "\n") {
                return Err(e);
            }
        }
        Ok(())
    }
}

//---

struct BasicGrid {
    size: Size,
    cells: Vec<Cell>,
}

impl BasicGrid {
    fn pos_to_index(&self, pos: UPos) -> usize {
        debug_assert!(pos.0 < self.width());
        debug_assert!(pos.1 < self.height());
        let idx = (pos.0 + pos.1 * self.width()) as usize;
        idx
    }
}

impl Grid for BasicGrid {
    fn size(&self) -> Size { self.size }
    fn get_cell(&self, pos: UPos) -> Cell {
        let idx = self.pos_to_index(pos);
        self.cells[idx]
    }
    fn set_cell(&mut self, pos: UPos, cell: Cell) {
        let idx = self.pos_to_index(pos);
        self.cells[idx] = cell;
    }
}

impl fmt::Display for BasicGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

//---

type BitGridRow = u16;

#[derive(Clone, Debug)]
struct BitGrid {
    size: Size,
    // cells: 0000000000
    // pos x: 9876543210
    rows: Vec<BitGridRow>,
    row_mask: BitGridRow,
}

impl BitGrid {
    fn new(size: Size) -> Self {
        debug_assert!(size.0 as usize <= std::mem::size_of::<BitGridRow>() * 8);
        Self {
            size,
            rows: vec![0; size.1 as usize],
            row_mask: !(!0 << (size.1 as BitGridRow)),
        }
    }
    fn put_fast(&mut self, pos: Pos, other: &BitGrid) -> bool {
        debug_assert!(self.width() >= other.width());
        debug_assert!(self.height() >= other.height());
        let mut dirty = false;
        let nshift = if pos.0 < 0 { -pos.0 } else { pos.0 } as BitGridRow;
        let to_right = pos.0 >= 0;
        let edge_checker = if to_right {
            1 << (self.width() - 1) as BitGridRow
        } else {
            1
        };
        for other_y in 0..other.height() {
            let mut row = other.rows[other_y as usize];
            if dirty {
                if to_right {
                    row <<= nshift;
                } else {
                    row >>= nshift;
                }
            } else {
                for _ in 0..nshift {
                    if row & edge_checker != 0 {
                        dirty = true;
                    }
                    if to_right {
                        row <<= 1;
                    } else {
                        row >>= 1;
                    }
                }
            }
            let y = pos.1 as usize + other_y as usize;
            if !dirty && self.rows[y] & row != 0 {
                dirty = true;
            }
            self.rows[y] |= row;
        }
        dirty
    }
    fn can_put_fast(&self, pos: Pos, other: &BitGrid) -> bool {
        debug_assert!(self.width() >= other.width());
        debug_assert!(self.height() >= other.height());
        let nshift = if pos.0 < 0 { -pos.0 } else { pos.0 } as BitGridRow;
        let to_right = pos.0 >= 0;
        let edge_checker = if to_right {
            1 << (self.width() - 1) as BitGridRow
        } else {
            1
        };
        for other_y in 0..other.height() {
            let mut row = other.rows[other_y as usize];
            for _ in 0..nshift {
                if row & edge_checker != 0 {
                    return false;
                }
                if to_right {
                    row <<= 1;
                } else {
                    row >>= 1;
                }
            }
            let y = pos.1 as usize + other_y as usize;
            if self.rows[y] & row != 0 {
                return false;
            }
        }
        true
    }
}

impl Grid for BitGrid {
    fn size(&self) -> Size { self.size }
    fn get_cell(&self, pos: UPos) -> Cell {
        debug_assert!(pos.0 < self.width());
        debug_assert!(pos.1 < self.height());
        let row = self.rows[pos.1 as usize];
        if row & (1 << pos.0) as BitGridRow != 0 {
            Cell::Block(Block::Any)
        } else {
            Cell::Empty
        }
    }
    fn set_cell(&mut self, pos: UPos, cell: Cell) {
        debug_assert!(pos.0 < self.width());
        debug_assert!(pos.1 < self.height());
        let row = self.rows[pos.1 as usize];
        self.rows[pos.1 as usize] = if !cell.is_empty() {
            row | (1 << pos.0) as BitGridRow
        } else {
            row & !((1 << pos.0) as BitGridRow)
        };
    }
    fn is_row_filled(&self, y: UPosY) -> bool {
        self.rows[y as usize] & self.row_mask == self.row_mask
    }
    fn is_row_empty(&self, y: UPosY) -> bool {
        self.rows[y as usize] & self.row_mask == 0
    }
    fn swap_rows(&mut self, y1: UPosY, y2: UPosY) {
        let r1 = self.rows[y1 as usize];
        self.rows[y1 as usize] = self.rows[y2 as usize];
        self.rows[y2 as usize] = r1;
    }
    fn set_row(&mut self, y: UPosY, cell: Cell) {
        let row = match cell {
            Cell::Empty => 0,
            _ => self.row_mask,
        };
        self.rows[y as usize] = row;
    }
}

impl fmt::Display for BitGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

//---

struct PieceSpec {
    basic_grids: Vec<BasicGrid>,
    initial_bit_grids: Vec<BitGrid>,
}

//---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_grid() {
        let mut grid = BitGrid::new((4, 5));
        assert!(!grid.has_cell((3, 1)));
        grid.set_cell((3, 1), Cell::Block(Block::Any));
        assert!(grid.has_cell((3, 1)));
        assert_eq!("    \n    \n    \n   @\n    ", format!("{}", grid));

        let mut grid2 = BitGrid::new((3, 2));
        grid2.set_cell((1, 0), Cell::Block(Block::Any));
        grid2.set_cell((2, 0), Cell::Block(Block::Any));
        assert!(grid.can_put((1, 0), &grid2));
        assert!(grid.can_put_fast((1, 0), &grid2));
        assert!(!grid.can_put((1, 1), &grid2));
        assert!(!grid.can_put_fast((1, 1), &grid2));
        assert!(grid.can_put((1, 2), &grid2));
        assert!(grid.can_put_fast((1, 2), &grid2));
        grid.put_fast((1, 0), &grid2);
        assert!(grid.has_cell((2, 0)));
        assert!(grid.has_cell((3, 0)));
    }
}
