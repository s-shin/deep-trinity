#[macro_use]
extern crate lazy_static;

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Orientation(u8);

pub const ORIENTATION_0: Orientation = Orientation(0);
pub const ORIENTATION_1: Orientation = Orientation(1);
pub const ORIENTATION_2: Orientation = Orientation(2);
pub const ORIENTATION_3: Orientation = Orientation(3);
pub const ORIENTATIONS: [Orientation; 4] = [ORIENTATION_0, ORIENTATION_1, ORIENTATION_2, ORIENTATION_3];

impl Orientation {
    pub fn new(n: u8) -> Self { Orientation(n % 4) }
    pub fn normalize(&mut self) {
        self.0 %= 4;
    }
    pub fn is(&self, n: u8) -> bool {
        debug_assert!(n < 4);
        self.0 % 4 == n
    }
    pub fn rotate(self, n: i8) -> Self {
        let mut n = (self.0 as i8 + n) % 4;
        if n < 0 {
            n += 4;
        }
        Self(n as u8)
    }
    pub fn id(self) -> u8 { self.0 % 4 }
}

//---

#[derive(Copy, Clone, Debug)]
pub enum Move {
    Shift(i8),
    Drop(i8),
    Rotate(i8),
}

impl Move {
    pub fn merge(self, m2: Move) -> Option<Move> {
        let m1 = self;
        match m2 {
            Move::Shift(n2) => {
                if n2 == 0 {
                    return Some(m1);
                }
                if let Move::Shift(n1) = m1 {
                    let n = n1 + n2;
                    if n != 0 {
                        return Some(Move::Shift(n));
                    }
                }
            }
            Move::Drop(n2) => {
                if n2 == 0 {
                    return Some(m1);
                }
                if let Move::Drop(n1) = m1 {
                    let n = n1 + n2;
                    if n != 0 {
                        return Some(Move::Drop(n));
                    }
                }
            }
            Move::Rotate(n2) => {
                if n2 == 0 {
                    return Some(m1);
                }
                // NOTE: Rotations cannot be merged.
            }
        }
        None
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MoveLogItem {
    pub by: Move,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
pub struct MoveLog {
    pub initial_pos: Pos,
    pub items: Vec<MoveLogItem>,
}

impl MoveLog {
    pub fn new(initial_pos: Pos) -> Self {
        Self {
            initial_pos,
            items: Vec::new(),
        }
    }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn push(&mut self, item: MoveLogItem) { self.items.push(item); }
    pub fn get(&self, i: usize) -> Option<&MoveLogItem> { self.items.get(i) }
    pub fn get_mut(&mut self, i: usize) -> Option<&mut MoveLogItem> { self.items.get_mut(i) }
    pub fn append(&mut self, other: &MoveLog) {
        if let Some(item) = self.items.last() {
            debug_assert_eq!(item.pos, other.initial_pos);
        }
        self.items.extend(&other.items);
    }
    pub fn merge_or_push(&mut self, item: MoveLogItem) {
        if let Some(last) = self.items.last_mut() {
            if let Some(mv) = last.by.merge(item.by) {
                last.by = mv;
                last.pos = item.pos;
                return;
            }
        }
        self.items.push(item);
    }
}

//---

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TSpin {
    Standard,
    Mini,
}

#[derive(Copy, Clone, Debug)]
pub struct LineClear {
    pub num_lines: u8,
    pub tspin: Option<TSpin>,
}

impl LineClear {
    pub fn new(num_lines: u8, tspin: Option<TSpin>) -> Self {
        Self { num_lines, tspin }
    }
    pub fn is_normal(&self) -> bool { self.tspin.is_none() }
    pub fn is_tetris(&self) -> bool { self.is_normal() && self.num_lines == 4 }
    pub fn is_tspin(&self) -> bool {
        if let Some(tspin) = self.tspin {
            tspin == TSpin::Standard
        } else {
            false
        }
    }
    pub fn is_tspin_mini(&self) -> bool {
        if let Some(tspin) = self.tspin {
            tspin == TSpin::Mini
        } else {
            false
        }
    }
}

//---

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum RotationMode {
    Srs,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum TSpinJudgementMode {
    PuyoPuyoTetris,
}

//---

// 0: Empty
// 1: Any
// 2-8: S, Z, L, J, I, T, O
// 9: Garbage
pub struct CellType(u8);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Piece {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Block {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
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

pub const CELL_CHARS: &'static str = " @SZLJITO#";

impl Cell {
    pub fn is_empty(self) -> bool {
        match self {
            Cell::Empty => true,
            _ => false
        }
    }
    pub fn char(self) -> char {
        let id: CellType = self.into();
        CELL_CHARS.chars().nth(id.0 as usize).unwrap()
    }
}

impl From<char> for Cell {
    fn from(c: char) -> Self {
        match c.to_ascii_uppercase() {
            ' ' => Cell::Empty,
            '@' => Cell::Block(Block::Any),
            'S' => Cell::Block(Block::Piece(Piece::S)),
            'Z' => Cell::Block(Block::Piece(Piece::Z)),
            'L' => Cell::Block(Block::Piece(Piece::L)),
            'J' => Cell::Block(Block::Piece(Piece::J)),
            'I' => Cell::Block(Block::Piece(Piece::I)),
            'T' => Cell::Block(Block::Piece(Piece::T)),
            'O' => Cell::Block(Block::Piece(Piece::O)),
            _ => Cell::Block(Block::Garbage),
        }
    }
}

//---

pub type SizeX = u8;
pub type SizeY = u8;
pub type Size = (SizeX, SizeY);
pub type PosX = i8;
pub type PosY = i8;
pub type Pos = (PosX, PosY);
pub type UPosX = SizeX;
pub type UPosY = SizeY;
pub type UPos = Size;

trait Grid: Clone + fmt::Display {
    fn size(&self) -> Size;
    fn width(&self) -> SizeX { self.size().0 }
    fn height(&self) -> SizeY { self.size().1 }
    fn get_cell(&self, pos: UPos) -> Cell;
    fn set_cell(&mut self, pos: UPos, cell: Cell);
    fn has_cell(&self, pos: UPos) -> bool { !self.get_cell(pos).is_empty() }
    fn put<G: Grid>(&mut self, pos: Pos, sub: &G) -> bool {
        let mut dirty = false;
        for sub_y in 0..sub.height() {
            for sub_x in 0..sub.width() {
                let sub_cell = sub.get_cell((sub_x, sub_y));
                if sub_cell.is_empty() {
                    continue;
                }
                let x = pos.0 + sub_x as i8;
                let y = pos.1 + sub_y as i8;
                if x < 0 || self.width() as i8 <= x || y < 0 || self.height() as i8 <= y {
                    dirty = true;
                    continue;
                }
                let p = (x as UPosX, y as UPosY);
                let cell = self.get_cell(p);
                if !cell.is_empty() {
                    dirty = true;
                }
                self.set_cell(p, sub_cell);
            }
        }
        !dirty
    }
    fn can_put<G: Grid>(&self, pos: Pos, sub: &G) -> bool {
        for sub_y in 0..sub.height() {
            for sub_x in 0..sub.width() {
                if !sub.has_cell((sub_x, sub_y)) {
                    continue;
                }
                let x = pos.0 + sub_x as i8;
                let y = pos.1 + sub_y as i8;
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
    fn num_filled_rows(&self) -> SizeY {
        let mut n = 0;
        for y in 0..self.height() {
            if self.is_row_filled(y) {
                n += 1;
            }
        }
        return n;
    }
    fn drop_filled_rows(&mut self) -> SizeY {
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
    fn num_droppable_rows<G: Grid>(&self, pos: Pos, sub: &G) -> SizeY {
        if !self.can_put(pos, sub) {
            return 0;
        }
        let mut n: SizeY = 1;
        while self.can_put((pos.0 - n as PosY, pos.1), sub) {
            n += 1;
        }
        n - 1
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BasicGrid {
    size: Size,
    cells: Vec<Cell>,
}

impl BasicGrid {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            cells: vec![Cell::Empty; size.0 as usize * size.1 as usize],
        }
    }
    fn pos_to_index(&self, pos: UPos) -> usize {
        debug_assert!(pos.0 < self.width());
        debug_assert!(pos.1 < self.height());
        let idx = pos.0 as usize + pos.1 as usize * self.width() as usize;
        idx
    }
    pub fn rotate_cw(&self) -> Self {
        let mut g = Self::new((self.height(), self.width()));
        for y in 0..self.height() {
            for x in 0..self.width() {
                g.set_cell((y, self.width() - 1 - x), self.get_cell((x, y)));
            }
        }
        g
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

#[derive(Clone, Debug, Eq)]
pub struct BitGrid {
    size: Size,
    // cells: 0000000000
    // pos x: 9876543210
    rows: Vec<BitGridRow>,
    row_mask: BitGridRow,
}

impl BitGrid {
    pub fn new(size: Size) -> Self {
        debug_assert!(size.0 as usize <= std::mem::size_of::<BitGridRow>() * 8);
        Self {
            size,
            rows: vec![0; size.1 as usize],
            row_mask: !(!0 << (size.0 as BitGridRow)),
        }
    }
    pub fn put_fast(&mut self, pos: Pos, sub: &BitGrid) -> bool {
        debug_assert!(self.width() >= sub.width());
        debug_assert!(self.height() >= sub.height());
        let mut dirty = false;
        let nshift = if pos.0 < 0 { -pos.0 } else { pos.0 } as BitGridRow;
        let to_right = pos.0 >= 0;
        let edge_checker = if to_right {
            1 << (self.width() - 1) as BitGridRow
        } else {
            1
        };
        for sub_y in 0..sub.height() {
            let mut row = sub.rows[sub_y as usize];
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
            let y = pos.1 as usize + sub_y as usize;
            if !dirty && self.rows[y] & row != 0 {
                dirty = true;
            }
            self.rows[y] |= row;
        }
        dirty
    }
    pub fn can_put_fast(&self, pos: Pos, sub: &BitGrid) -> bool {
        debug_assert!(self.width() >= sub.width());
        debug_assert!(self.height() >= sub.height());
        let nshift = pos.0.abs() as BitGridRow;
        let to_right = pos.0 >= 0;
        let edge_checker = if to_right {
            1 << (self.width() - 1) as BitGridRow
        } else {
            1
        };
        for sub_y in 0..sub.height() {
            let mut row = sub.rows[sub_y as usize];
            let y = pos.1 + sub_y as PosY;
            if y < 0 || y >= self.height() as PosY {
                if row != 0 {
                    return false;
                }
                continue;
            }
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
            if self.rows[y as usize] & row != 0 {
                return false;
            }
        }
        true
    }
    pub fn num_droppable_rows_fast(&self, pos: Pos, sub: &BitGrid) -> SizeY {
        if !self.can_put_fast(pos, sub) {
            return 0;
        }
        let mut rows_cache: Vec<BitGridRow> = Vec::with_capacity(sub.height() as usize);
        let to_right = pos.0 > 0;
        for row in &sub.rows {
            rows_cache.push(if to_right {
                *row << pos.0
            } else {
                *row >> (-pos.0)
            })
        }
        let mut n: SizeY = 1;
        loop {
            let mut can_put = true;
            for sub_y in 0..sub.height() {
                let row = rows_cache[sub_y as usize];
                let y = pos.1 as PosY - n as PosY + sub_y as PosY;
                if y < 0 || y >= self.height() as PosY {
                    if row != 0 {
                        can_put = false;
                        break;
                    }
                    continue;
                }
                if self.rows[y as usize] & rows_cache[sub_y as usize] != 0 {
                    can_put = false;
                    break;
                }
            }
            if can_put {
                n += 1;
            } else {
                break;
            }
        }
        n - 1
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

impl PartialEq for BitGrid {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.rows == other.rows
    }
}

impl Hash for BitGrid {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.size.hash(state);
        self.rows.hash(state);
    }
}


//---

#[derive(Clone, Debug, Eq)]
pub struct HybridGrid {
    pub basic_grid: BasicGrid,
    pub bit_grid: BitGrid,
}

impl HybridGrid {
    pub fn new(size: Size) -> Self {
        Self {
            basic_grid: BasicGrid::new(size),
            bit_grid: BitGrid::new(size),
        }
    }
    pub fn with_grids(basic_grid: BasicGrid, bit_grid: BitGrid) -> Self {
        debug_assert_eq!(basic_grid.size(), bit_grid.size());
        Self {
            basic_grid,
            bit_grid,
        }
    }
    pub fn put_fast(&mut self, pos: Pos, sub: &BitGrid) {
        self.basic_grid.put(pos, sub);
        self.bit_grid.put_fast(pos, sub);
    }
}

impl Grid for HybridGrid {
    fn size(&self) -> Size { self.basic_grid.size() }
    fn get_cell(&self, pos: UPos) -> Cell { self.basic_grid.get_cell(pos) }
    fn set_cell(&mut self, pos: UPos, cell: Cell) {
        self.basic_grid.set_cell(pos, cell);
        self.bit_grid.set_cell(pos, cell);
    }
    fn is_row_filled(&self, y: UPosY) -> bool { self.bit_grid.is_row_filled(y) }
    fn is_row_empty(&self, y: UPosY) -> bool { self.bit_grid.is_row_empty(y) }
    fn swap_rows(&mut self, y1: UPosY, y2: UPosY) {
        self.basic_grid.swap_rows(y1, y2);
        self.bit_grid.swap_rows(y1, y2);
    }
    fn set_row(&mut self, y: UPosY, cell: Cell) {
        self.basic_grid.set_row(y, cell);
        self.bit_grid.set_row(y, cell);
    }
}

impl fmt::Display for HybridGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

impl PartialEq for HybridGrid {
    fn eq(&self, other: &Self) -> bool { self.bit_grid == other.bit_grid }
}

impl Hash for HybridGrid {
    fn hash<H: Hasher>(&self, state: &mut H) { self.bit_grid.hash(state); }
}

//---

fn srs_offset_data_i() -> Vec<Vec<Pos>> {
    vec![
        vec![(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        vec![(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        vec![(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        vec![(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
    ]
}

fn srs_offset_data_o() -> Vec<Vec<Pos>> {
    vec![
        vec![(0, 0)],
        vec![(0, -1)],
        vec![(-1, -1)],
        vec![(-1, 0)],
    ]
}

fn srs_offset_data_others() -> Vec<Vec<Pos>> {
    vec![
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    ]
}

pub struct PieceSpec {
    // The index of Vec is orientation.
    grids: Vec<HybridGrid>,
    initial_pos: Pos,
    // The index of outer Vec is orientation.
    srs_offset_data: Vec<Vec<Pos>>,
}

impl PieceSpec {
    fn new(piece: Piece, size: Size, block_pos_list: Vec<UPos>, initial_pos: Pos,
           srs_offset_data: Vec<Vec<Pos>>) -> Self {
        let piece_cell = Cell::Block(Block::Piece(piece));
        let mut grid = BasicGrid::new(size);
        for pos in block_pos_list {
            grid.set_cell(pos, piece_cell);
        }
        let grid_deg90 = grid.rotate_cw();
        let grid_deg180 = grid_deg90.rotate_cw();
        let grid_deg270 = grid_deg180.rotate_cw();
        let basic_grids = vec![
            grid,
            grid_deg90,
            grid_deg180,
            grid_deg270,
        ];
        let mut grids: Vec<HybridGrid> = Vec::with_capacity(basic_grids.len());
        for basic_grid in basic_grids {
            let mut bit_grid = BitGrid::new(basic_grid.size());
            bit_grid.put((0, 0), &basic_grid);
            grids.push(HybridGrid::with_grids(basic_grid, bit_grid));
        }
        Self {
            grids,
            initial_pos,
            srs_offset_data,
        }
    }

    fn piece_s() -> Self {
        Self::new(
            Piece::S,
            (3, 3),
            vec![(0, 1), (1, 1), (1, 2), (2, 2)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    fn piece_z() -> Self {
        Self::new(
            Piece::Z,
            (3, 3),
            vec![(0, 2), (1, 1), (1, 2), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    fn piece_l() -> Self {
        Self::new(
            Piece::L,
            (3, 3),
            vec![(0, 1), (1, 1), (2, 1), (2, 2)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    fn piece_j() -> Self {
        Self::new(
            Piece::J,
            (3, 3),
            vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    fn piece_i() -> Self {
        Self::new(
            Piece::I,
            (5, 5),
            vec![(1, 2), (2, 2), (3, 2), (4, 2)],
            (3, 17),
            srs_offset_data_i(),
        )
    }
    fn piece_t() -> Self {
        Self::new(
            Piece::T,
            (3, 3),
            vec![(0, 1), (1, 1), (1, 2), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    fn piece_o() -> Self {
        Self::new(
            Piece::T,
            (3, 3),
            vec![(1, 1), (1, 2), (2, 1), (2, 2)],
            (3, 18),
            srs_offset_data_o(),
        )
    }
    pub fn of(piece: Piece) -> &'static Self { &PIECE_SPECS[piece as usize] }
}

fn gen_piece_specs() -> Vec<PieceSpec> {
    vec![
        PieceSpec::piece_s(),
        PieceSpec::piece_z(),
        PieceSpec::piece_l(),
        PieceSpec::piece_j(),
        PieceSpec::piece_i(),
        PieceSpec::piece_t(),
        PieceSpec::piece_o(),
    ]
}

lazy_static! {
    pub static ref PIECE_SPECS: Vec<PieceSpec> = gen_piece_specs();
}

//---

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PieceLocation {
    orientation: Orientation,
    pos: Pos,
}

impl PieceLocation {
    fn new(orientation: Orientation, pos: Pos) -> Self {
        Self { orientation, pos }
    }
}

#[derive(Clone, Debug)]
pub struct FallingPiece {
    pub piece: Piece,
    pub loc: PieceLocation,
    pub move_log: MoveLog,
}

impl FallingPiece {
    pub fn new(piece: Piece, loc: PieceLocation) -> Self {
        Self { piece, loc, move_log: MoveLog::new(loc.pos) }
    }
    pub fn spawn(piece: Piece) -> Self {
        let spec = PieceSpec::of(piece);
        Self::new(piece, PieceLocation::new(ORIENTATION_0, spec.initial_pos))
    }
    pub fn grid(&self) -> &'static HybridGrid {
        &PieceSpec::of(self.piece).grids[self.loc.orientation.id() as usize]
    }
    pub fn apply_move(&mut self, mv: Move, pf: &Playfield, mode: RotationMode) -> bool {
        debug_assert_eq!(RotationMode::Srs, mode);
        let pf_str = format!("{}", pf.grid);
        match mv {
            Move::Shift(n) => {
                if !pf.can_move_horizontally(self, n) {
                    return false;
                }
                self.loc.pos.0 += n;
            }
            Move::Drop(n) => {
                if !pf.can_drop_n(self, n as SizeY) {
                    return false;
                }
                self.loc.pos.1 -= n
            }
            Move::Rotate(n) => {
                let backup = self.loc;
                for _ in 0..n.abs() {
                    if let Some(pos) = pf.check_rotation_by_srs(self, n > 0) {
                        self.loc.orientation = self.loc.orientation.rotate(if n > 0 { 1 } else { -1 });
                        self.loc.pos = pos;
                    } else {
                        self.loc = backup;
                        return false;
                    }
                }
            }
        }
        self.move_log.push(MoveLogItem { by: mv, pos: self.loc.pos });
        true
    }
    pub fn rollback(&mut self) -> bool {
        if let Some(removed) = self.move_log.items.pop() {
            self.loc.pos = if let Some(last) = self.move_log.items.last() {
                last.pos
            } else {
                self.move_log.initial_pos
            };
            if let Move::Rotate(n) = removed.by {
                self.loc.orientation.rotate(-n);
            }
            true
        } else {
            false
        }
    }
    pub fn is_last_move_rotation(&self) -> bool {
        if let Some(item) = self.move_log.items.last() {
            if let Move::Rotate(_) = item.by {
                return true;
            }
        }
        false
    }
    pub fn last_two_positions(&self) -> Option<(Pos, Pos)> {
        let len = self.move_log.items.len();
        if len < 2 {
            return None;
        }
        Some((
            self.move_log.get(len - 2).unwrap().pos,
            self.move_log.get(len - 1).unwrap().pos,
        ))
    }
}

//---

pub const PLAYFIELD_SIZE: Size = (10, 40);
pub const PLAYFIELD_VISIBLE_HEIGHT: SizeY = 20;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Playfield {
    pub grid: HybridGrid,
}

impl Playfield {
    pub fn new() -> Self {
        Self {
            grid: HybridGrid::new(PLAYFIELD_SIZE),
        }
    }
    pub fn set_rows(&mut self, pos: UPos, rows: &[&'static str]) {
        for (dy, row) in rows.iter().rev().enumerate() {
            let y = pos.1 + dy as UPosY;
            if y >= self.grid.height() {
                return;
            }
            for (dx, c) in row.chars().enumerate() {
                let x = pos.0 + dx as UPosX;
                if x >= self.grid.width() {
                    break;
                }
                self.grid.set_cell((x, y), c.into());
            }
        }
    }
    pub fn can_put(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put(fp.loc.pos, &fp.grid().bit_grid)
    }
    pub fn num_droppable_rows(&self, fp: &FallingPiece) -> SizeY {
        self.grid.bit_grid.num_droppable_rows_fast(fp.loc.pos, &fp.grid().bit_grid)
    }
    pub fn can_drop(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put((fp.loc.pos.0, fp.loc.pos.1 - 1), &fp.grid().bit_grid)
    }
    pub fn can_drop_n(&self, fp: &FallingPiece, n: SizeY) -> bool {
        n <= self.grid.bit_grid.num_droppable_rows_fast(fp.loc.pos, &fp.grid().bit_grid)
    }
    pub fn can_move_horizontally(&self, fp: &FallingPiece, n: PosX) -> bool {
        let to_right = n > 0;
        let end = if to_right { n } else { -n };
        for dx in 0..end {
            let x = fp.loc.pos.0 + if to_right { dx } else { -dx };
            if !self.grid.bit_grid.can_put_fast((x, fp.loc.pos.1), &fp.grid().bit_grid) {
                return false;
            }
        }
        true
    }
    pub fn check_rotation_by_srs(&self, fp: &FallingPiece, cw: bool) -> Option<Pos> {
        let after: Orientation = fp.loc.orientation.rotate(if cw { 1 } else { -1 });
        let spec = PieceSpec::of(fp.piece);
        let next_grid: &HybridGrid = &spec.grids[after.id() as usize];
        let offsets1: &Vec<Pos> = &spec.srs_offset_data[fp.loc.orientation.id() as usize];
        let offsets2: &Vec<Pos> = &spec.srs_offset_data[after.id() as usize];
        for i in 0..offsets1.len() {
            let (mut x, mut y) = fp.loc.pos;
            x += offsets1[i].0 - offsets2[i].0;
            y += offsets1[i].1 - offsets2[i].1;
            if self.grid.bit_grid.can_put_fast((x, y), &next_grid.bit_grid) {
                return Some((x, y));
            }
        }
        None
    }
    pub fn can_lock(&self, fp: &FallingPiece) -> bool { self.can_put(fp) && !self.can_drop(fp) }
    pub fn check_tspin(&self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<TSpin> {
        debug_assert!(self.can_lock(fp));
        debug_assert_eq!(TSpinJudgementMode::PuyoPuyoTetris, mode);
        if fp.piece != Piece::T || !fp.is_last_move_rotation() {
            return None;
        }
        let mut num_corners = 0;
        let mut num_pointing_side_corners = 0;
        for dy in &[0, 2] {
            for dx in &[0, 2] {
                let dx = *dx;
                let dy = *dy;
                let pos = (fp.loc.pos.0 + dx, fp.loc.pos.1 + dy);
                let is_wall = pos.0 < 0 || pos.1 < 0;
                if is_wall || self.grid.has_cell((pos.0 as UPosX, pos.1 as UPosY)) {
                    num_corners += 1;
                    if match fp.loc.orientation {
                        ORIENTATION_0 => { (dx, dy) == (0, 2) || (dx, dy) == (2, 2) }
                        ORIENTATION_1 => { (dx, dy) == (2, 0) || (dx, dy) == (2, 2) }
                        ORIENTATION_2 => { (dx, dy) == (0, 0) || (dx, dy) == (2, 0) }
                        ORIENTATION_3 => { (dx, dy) == (0, 0) || (dx, dy) == (0, 2) }
                        _ => panic!(),
                    } {
                        num_pointing_side_corners += 1;
                    }
                }
            }
        }
        match num_corners {
            3 => {
                if num_pointing_side_corners == 2 {
                    Some(TSpin::Standard)
                } else if let Some((p1, p2)) = fp.last_two_positions() {
                    let is_shifted = p1.0 != p2.0;
                    let num_rows = p1.0 - p2.0;
                    if num_rows == 2 {
                        if is_shifted {
                            Some(TSpin::Standard)
                        } else {
                            Some(TSpin::Mini) // Neo
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            4 => Some(TSpin::Standard),
            _ => None,
        }
    }
    pub fn check_line_clear(&self, fp: &FallingPiece, mode: TSpinJudgementMode) -> LineClear {
        debug_assert!(self.can_lock(fp));
        let mut tmp_grid = self.grid.clone();
        tmp_grid.put_fast(fp.loc.pos, &fp.grid().bit_grid);
        LineClear::new(tmp_grid.num_filled_rows(), self.check_tspin(fp, mode))
    }
    pub fn lock(&mut self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<LineClear> {
        if !self.can_lock(fp) {
            return None;
        }
        let tspin = self.check_tspin(fp, mode);
        self.grid.put_fast(fp.loc.pos, &fp.grid().bit_grid);
        let num_cleared_line = self.grid.drop_filled_rows();
        Some(LineClear::new(num_cleared_line, tspin))
    }
    // The return locations can include unreachable locations.
    pub fn search_lockable_locations(&self, piece: Piece) -> Vec<PieceLocation> {
        let yend = (self.grid.height() - self.grid.top_padding()) as PosY;
        let spec = PieceSpec::of(piece);
        let sub_bit_grids = [
            &spec.grids[ORIENTATION_0.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_1.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_2.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_3.id() as usize].bit_grid,
        ];
        let mut r: Vec<PieceLocation> = Vec::new();
        for y in -1..yend {
            for x in -1..(self.grid.width() as PosX - 1) {
                for o in &ORIENTATIONS {
                    let g = sub_bit_grids[o.id() as usize];
                    let can_put = self.grid.bit_grid.can_put_fast((x, y), g);
                    if !can_put {
                        continue;
                    }
                    let can_drop = self.grid.bit_grid.can_put_fast((x, y - 1), g);
                    if can_drop {
                        continue;
                    }
                    r.push(PieceLocation::new(*o, (x, y)));
                }
            }
        }
        r
    }
}

// mod basic_route_finder_aaa {
//     use super::*;
//     use std::collections::{HashMap, HashSet};
//
//     macro_rules! debug_println {
//         ($self:ident, $e:expr $(, $es:expr)*) => {
//             if $self.debug {
//                 if $self.depth > 0 {
//                     print!("{}", "│".repeat($self.depth));
//                 }
//                 println!($e $(, $es)*);
//             }
//         }
//     }
//
//     // [1] horizontal move
//     // [2] rotation
//     // [3] drop to bottom
//     // [4] rotation
//     // first search: horizontally -> rotation -> vertically -> rotation
//     // rest searches: vertical (depth-first) -> horizontal -> rotation
//
//     pub struct BasicRouteFinder<'a> {
//         pf: &'a Playfield,
//         piece: Piece,
//         src: PieceLocation,
//         dsts: HashSet<PieceLocation>,
//         mode: RotationMode,
//         debug: bool,
//         checked: HashMap<PieceLocation, MoveLogItem>,
//         all_checked: HashSet<PieceLocation>,
//         depth: usize,
//         results: Vec<Option<MoveLog>>,
//         move_order: [Move; 5],
//     }
//
//     impl<'a> BasicRouteFinder<'a> {
//         pub fn new(pf: &'a Playfield, piece: Piece, src: PieceLocation, mode: RotationMode, debug: bool) -> Self {
//             Self {
//                 pf,
//                 piece,
//                 src,
//                 dsts: HashSet::new(),
//                 mode,
//                 debug,
//                 checked: HashMap::new(),
//                 all_checked: HashSet::new(),
//                 depth: 0,
//                 results: vec![None; dsts.len()],
//                 move_order: [Move::Shift(1), Move::Drop(1), Move::Rotate(1), Move::Rotate(-1), Move::Shift(-1)],
//             }
//         }
//         pub fn find(&mut self, dsts: HashSet<PieceLocation>) -> Vec<Option<MoveLog>> {
//             self.dsts = dsts;
//             self.depth = 0;
//             self.results.clear();
//
//             // let mut pos = dst;
//             // let mut items = vec![];
//             // loop {
//             //     let target = &PieceLocation::new(self.src.orientation, pos);
//             //     if let Some(item) = self.check_sheet.get(&target) {
//             //         items.push(item);
//             //         if item.pos == self.src.pos {
//             //             return Some(log);
//             //         }
//             //         pos = item.pos;
//             //     } else {
//             //         break;
//             //     }
//             // }
//             self.search_all(&FallingPiece::new(self.piece, self.src));
//
//             let mut r: Vec<Option<MoveLog>> = Vec::new();
//             r.append(&mut self.results);
//             r
//         }
//         fn search_all(&mut self, fp: &FallingPiece) -> Option<MoveLog> {
//             debug_println!(self, "search_all: {:?} ({}, {})", fp.loc.orientation, fp.loc.pos.0, fp.loc.pos.1);
//             if self.all_checked.contains(&fp.loc) {
//                 debug_println!(self, "=> already checked.");
//                 return None;
//             }
//             self.last_checked = Some(fp.loc.clone());
//             let ok = self.check_sheet.insert(fp.loc);
//             debug_assert!(ok);
//
//             let mut fp = fp.clone();
//             if self.dsts.remove(&fp.loc) {
//                 debug_println!(self, "=> found!");
//                 return Some(MoveLog::new(fp.loc.pos));
//             }
//
//             for mv in &self.move_order {
//                 let mv_str = format!("{:?}", mv);
//                 debug_println!(self, "├ {:?}", mv);
//                 if fp.apply_move(*mv, self.pf, self.mode) {
//                     self.depth += 1;
//                     let search_fp = FallingPiece::new(fp.piece, fp.loc);
//                     if let Some(hereafter) = self.search_all(&search_fp) {
//                         fp.move_log.append(&hereafter);
//                         self.depth -= 1;
//                         return Some(fp.move_log);
//                     }
//                     fp.rollback();
//                     self.depth -= 1;
//                 }
//             }
//
//             debug_println!(self, "=> not found.");
//             None
//         }
//     }
// }

struct SearchMovableLocationsResult {
    found: HashMap<PieceLocation, MoveLogItem>,
}

impl SearchMovableLocationsResult {
    fn get(&self, dst: &PieceLocation) -> Option<MoveLog> {
        None
    }
}

fn search_movable_locations(pf: &Playfield, piece: Piece, src: PieceLocation, mode: RotationMode,
                            debug: bool) -> SearchMovableLocationsResult {
    struct Configuration<'a> {
        pf: &'a Playfield,
        mode: RotationMode,
        debug: bool,
        move_order: [Move; 5],
    }

    let mut conf = Configuration {
        pf,
        mode,
        debug,
        move_order: [Move::Shift(1), Move::Drop(1), Move::Rotate(1), Move::Rotate(-1), Move::Shift(-1)],
    };

    type Found = HashMap<PieceLocation, MoveLogItem>;
    let mut found: Found = HashMap::new();

    fn search(conf: &Configuration, fp: &FallingPiece, depth: usize, found: &mut Found) {
        macro_rules! debug_println {
            ($e:expr $(, $es:expr)*) => {
                if conf.debug {
                    if depth > 0 {
                        print!("{}", "│".repeat(depth));
                    }
                    println!($e $(, $es)*);
                }
            }
        }

        debug_println!("search_all: {:?} ({}, {})", fp.loc.orientation, fp.loc.pos.0, fp.loc.pos.1);
        if found.contains_key(&fp.loc) {
            debug_println!("=> already checked.");
            return;
        }
        if let Some(last) = fp.move_log.items.last() {
            let v = found.insert(fp.loc, *last);
            debug_assert!(v.is_none());
        }

        let mut fp = FallingPiece::new(fp.piece, fp.loc);
        for mv in &conf.move_order {
            debug_println!("├ {:?}", mv);
            if fp.apply_move(*mv, conf.pf, conf.mode) {
                search(conf, &fp, depth + 1, found);
                fp.rollback();
            }
        }

        debug_println!("=> checked.");
    };

    search(&mut conf, &FallingPiece::new(piece, src), 0, &mut found);

    SearchMovableLocationsResult { found }
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

    #[test]
    fn test_search_droppable_routes() {
        let mut pf = Playfield::new();
        pf.set_rows((0, 0), &[
            "          ",
            "          ",
            "@@        ",
            "@         ",
            "@ @@@@    ",
            "@   @@    ",
            "@    @    ",
            "@    @    ",
            "@@  @     ",
            "@   @     ",
            "@ @@@     ",
            "@  @@     ",
            "@   @     ",
            "@@@ @     ",
            "@@  @     ",
            "@   @     ",
            "@ @@@     ",
            "@  @@     ",
            "@   @     ",
            "@@ @@@    ",
        ]);
        let fp = FallingPiece::spawn(Piece::T);
        let r = search_movable_locations(&pf, fp.piece, fp.loc, RotationMode::Srs, false);
        let move_log = r.get(&PieceLocation::new(ORIENTATION_3, (1, 0)));
        assert!(move_log.is_some());
        // let finder = BasicRouteFinder::new(RotationMode::Srs);
        // let routes = pf.search_droppable_routes(&fp, &finder);
        // let mut found = false;
        // for route in routes {
        //     if route.orientation == ORIENTATION_3 && route.pos == (1, 0) {
        //         found = true;
        //         break;
        //     }
        // }
        // assert!(found);
    }
}
