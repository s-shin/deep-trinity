#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;

//---

pub type PosX = i8;
pub type PosY = i8;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pos(PosX, PosY);

macro_rules! pos {
    ($x:expr, $y:expr) => { Pos($x, $y) }
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({}, {})", self.0, self.1) }
}

impl From<(PosX, PosY)> for Pos {
    fn from(pos: (PosX, PosY)) -> Self { Self(pos.0, pos.1) }
}

impl ops::Add for Pos {
    type Output = Self;
    fn add(self, other: Self) -> Self { Self(self.0 + other.0, self.1 + other.1) }
}

impl ops::Sub for Pos {
    type Output = Self;
    fn sub(self, other: Self) -> Self { Self(self.0 - other.0, self.1 - other.1) }
}

pub type UPosX = u8;
pub type UPosY = u8;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct UPos(SizeX, SizeY);

macro_rules! upos {
    ($x:expr, $y:expr) => { UPos($x, $y) }
}

impl fmt::Display for UPos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({}, {})", self.0, self.1) }
}

impl From<(UPosX, UPosY)> for UPos {
    fn from(pos: (UPosX, UPosY)) -> Self { Self(pos.0, pos.1) }
}

impl ops::Add for UPos {
    type Output = Self;
    fn add(self, other: Self) -> Self { Self(self.0 + other.0, self.1 + other.1) }
}

impl ops::Sub for UPos {
    type Output = Self;
    fn sub(self, other: Self) -> Self { Self(self.0 - other.0, self.1 - other.1) }
}

pub type SizeX = UPosX;
pub type SizeY = UPosY;
pub type Size = UPos;

macro_rules! size {
    ($x:expr, $y:expr) => { UPos($x, $y) }
}

// Pos -> UPos
impl From<Pos> for UPos {
    fn from(pos: Pos) -> Self {
        debug_assert!(pos.0 >= 0);
        debug_assert!(pos.1 >= 0);
        Self(pos.0 as SizeX, pos.1 as SizeY)
    }
}

// UPos -> Pos
impl From<UPos> for Pos {
    fn from(pos: UPos) -> Self {
        debug_assert!(pos.0 <= PosX::max_value() as UPosX);
        debug_assert!(pos.1 <= PosY::max_value() as UPosY);
        Self(pos.0 as PosX, pos.1 as PosY)
    }
}

// UPos + Pos -> UPos
impl ops::Add<Pos> for UPos {
    type Output = Self;
    fn add(self, other: Pos) -> Self { Self::from(Pos::from(self) + other) }
}

// Pos + UPos -> Pos
impl ops::Add<UPos> for Pos {
    type Output = Self;
    fn add(self, other: UPos) -> Self { self + Self::from(other) }
}

//---

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
                let sub_pos = upos!(sub_x, sub_y);
                let sub_cell = sub.get_cell(sub_pos);
                if sub_cell.is_empty() {
                    continue;
                }
                let p = pos + sub_pos;
                if p.0 < 0 || self.width() as i8 <= p.0 || p.1 < 0 || self.height() as i8 <= p.1 {
                    dirty = true;
                    continue;
                }
                let p = UPos::from(p);
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
                let sub_pos = upos!(sub_x, sub_y);
                if !sub.has_cell(sub_pos) {
                    continue;
                }
                let p = pos + sub_pos;
                if p.0 < 0 || self.width() as i8 <= p.0 || p.1 < 0 || self.height() as i8 <= p.1 {
                    return false;
                }
                if self.has_cell(p.into()) {
                    return false;
                }
            }
        }
        true
    }
    fn is_row_filled(&self, y: UPosY) -> bool {
        for x in 0..self.width() {
            if !self.has_cell(upos!(x, y)) {
                return false;
            }
        }
        true
    }
    fn is_row_empty(&self, y: UPosY) -> bool {
        for x in 0..self.width() {
            if self.has_cell(upos!(x, y)) {
                return false;
            }
        }
        true
    }
    fn is_col_filled(&self, x: UPosX) -> bool {
        for y in 0..self.height() {
            if !self.has_cell(upos!(x, y)) {
                return false;
            }
        }
        true
    }
    fn is_col_empty(&self, x: UPosX) -> bool {
        for y in 0..self.height() {
            if self.has_cell(upos!(x, y)) {
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
            let c1 = self.get_cell(upos!(x, y1));
            let c2 = self.get_cell(upos!(x, y2));
            self.set_cell(upos!(x, y1), c2);
            self.set_cell(upos!(x, y2), c1);
        }
    }
    fn set_row(&mut self, y: UPosY, cell: Cell) {
        debug_assert!(y < self.height());
        for x in 0..self.width() {
            self.set_cell(upos!(x, y), cell);
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
        while self.can_put(pos!(pos.0 - n as PosY, pos.1), sub) {
            n += 1;
        }
        n - 1
    }
    fn format(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for y in (0..self.height()).rev() {
            for x in 0..self.width() {
                let c = self.get_cell(upos!(x, y)).char();
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
        let mut g = Self::new(size!(self.height(), self.width()));
        for y in 0..self.height() {
            for x in 0..self.width() {
                g.set_cell(upos!(y, self.width() - 1 - x), self.get_cell(upos!(x, y)));
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

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Placement {
    orientation: Orientation,
    pos: Pos,
}

impl Placement {
    fn new(orientation: Orientation, pos: Pos) -> Self {
        Self { orientation, pos }
    }
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
pub struct MoveRecordItem {
    pub by: Move,
    pub placement: Placement,
}

impl MoveRecordItem {
    fn new(by: Move, placement: Placement) -> Self {
        Self { by, placement }
    }
}

#[derive(Clone, Debug)]
pub struct MoveRecord {
    pub initial_placement: Placement,
    pub items: Vec<MoveRecordItem>,
}

impl MoveRecord {
    pub fn new(initial_placement: Placement) -> Self {
        Self {
            initial_placement,
            items: Vec::new(),
        }
    }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn push(&mut self, item: MoveRecordItem) { self.items.push(item); }
    pub fn pop(&mut self) -> Option<MoveRecordItem> { self.items.pop() }
    pub fn get(&self, i: usize) -> Option<&MoveRecordItem> { self.items.get(i) }
    pub fn get_mut(&mut self, i: usize) -> Option<&mut MoveRecordItem> { self.items.get_mut(i) }
    pub fn last(&self) -> Option<&MoveRecordItem> { self.items.last() }
    pub fn append(&mut self, other: &MoveRecord) {
        if let Some(item) = self.items.last() {
            debug_assert_eq!(item.placement, other.initial_placement);
        }
        self.items.extend(&other.items);
    }
    pub fn merge_or_push(&mut self, item: MoveRecordItem) {
        if let Some(last) = self.items.last_mut() {
            if let Some(mv) = last.by.merge(item.by) {
                last.by = mv;
                last.placement = item.placement;
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

fn srs_offset_data_i() -> Vec<Vec<(PosX, PosY)>> {
    vec![
        vec![(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        vec![(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        vec![(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        vec![(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
    ]
}

fn srs_offset_data_o() -> Vec<Vec<(PosX, PosY)>> {
    vec![
        vec![(0, 0)],
        vec![(0, -1)],
        vec![(-1, -1)],
        vec![(-1, 0)],
    ]
}

fn srs_offset_data_others() -> Vec<Vec<(PosX, PosY)>> {
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
    initial_placement: Placement,
    // The index of outer Vec is orientation.
    srs_offset_data: Vec<Vec<(PosX, PosY)>>,
}

impl PieceSpec {
    fn new(piece: Piece, size: (SizeX, SizeY), block_pos_list: Vec<(UPosX, UPosY)>,
           initial_pos: (PosX, PosY), srs_offset_data: Vec<Vec<(PosX, PosY)>>) -> Self {
        let piece_cell = Cell::Block(Block::Piece(piece));
        let mut grid = BasicGrid::new(size.into());
        for pos in block_pos_list {
            grid.set_cell(pos.into(), piece_cell);
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
            bit_grid.put(pos!(0, 0), &basic_grid);
            grids.push(HybridGrid::with_grids(basic_grid, bit_grid));
        }
        Self {
            grids,
            initial_placement: Placement::new(ORIENTATION_0, initial_pos.into()),
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

#[derive(Clone, Debug)]
pub struct FallingPiece {
    pub piece: Piece,
    pub placement: Placement,
    pub move_record: MoveRecord,
}

impl FallingPiece {
    pub fn new(piece: Piece, placement: Placement) -> Self {
        Self { piece, placement, move_record: MoveRecord::new(placement) }
    }
    pub fn spawn(piece: Piece, pf: Option<&Playfield>) -> Self {
        let spec = PieceSpec::of(piece);
        let mut fp = Self::new(piece, spec.initial_placement);
        if let Some(pf) = pf {
            if !pf.can_put(&fp) {
                fp.placement.pos.0 -= 1;
                fp.move_record.initial_placement.pos.0 -= 1;
            }
            debug_assert!(pf.can_put(&fp));
        }
        fp
    }
    pub fn grid(&self) -> &'static HybridGrid {
        &PieceSpec::of(self.piece).grids[self.placement.orientation.id() as usize]
    }
    pub fn apply_move(&mut self, mv: Move, pf: &Playfield, mode: RotationMode) -> bool {
        debug_assert_eq!(RotationMode::Srs, mode);
        match mv {
            Move::Shift(n) => {
                if !pf.can_move_horizontally(self, n) {
                    return false;
                }
                self.placement.pos.0 += n;
            }
            Move::Drop(n) => {
                if !pf.can_drop_n(self, n as SizeY) {
                    return false;
                }
                self.placement.pos.1 -= n
            }
            Move::Rotate(n) => {
                let backup = self.placement;
                for _ in 0..n.abs() {
                    if let Some(p) = pf.check_rotation_by_srs(self, n > 0) {
                        self.placement = p;
                    } else {
                        self.placement = backup;
                        return false;
                    }
                }
            }
        }
        self.move_record.push(MoveRecordItem::new(mv, self.placement));
        true
    }
    pub fn rollback(&mut self) -> bool {
        if let Some(_) = self.move_record.pop() {
            self.placement = self.move_record.last()
                .map_or(self.move_record.initial_placement, |item| { item.placement });
            true
        } else {
            false
        }
    }
    pub fn is_last_move_rotation(&self) -> bool {
        if let Some(item) = self.move_record.items.last() {
            if let Move::Rotate(_) = item.by {
                return true;
            }
        }
        false
    }
    pub fn last_two_placements(&self) -> Option<(Placement, Placement)> {
        let len = self.move_record.items.len();
        if len < 2 {
            return None;
        }
        Some((
            self.move_record.get(len - 2).unwrap().placement,
            self.move_record.get(len - 1).unwrap().placement,
        ))
    }
}

//---

pub const PLAYFIELD_SIZE: Size = size!(10, 40);
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
                self.grid.set_cell(upos!(x, y), c.into());
            }
        }
    }
    pub fn can_put(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put(fp.placement.pos, &fp.grid().bit_grid)
    }
    pub fn num_droppable_rows(&self, fp: &FallingPiece) -> SizeY {
        self.grid.bit_grid.num_droppable_rows_fast(fp.placement.pos, &fp.grid().bit_grid)
    }
    pub fn can_drop(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put(fp.placement.pos + pos!(0, -1), &fp.grid().bit_grid)
    }
    pub fn can_drop_n(&self, fp: &FallingPiece, n: SizeY) -> bool {
        n <= self.grid.bit_grid.num_droppable_rows_fast(fp.placement.pos, &fp.grid().bit_grid)
    }
    pub fn can_move_horizontally(&self, fp: &FallingPiece, n: PosX) -> bool {
        let to_right = n > 0;
        let end = if to_right { n } else { -n };
        for dx in 0..end {
            let x = fp.placement.pos.0 + if to_right { dx } else { -dx };
            if !self.grid.bit_grid.can_put_fast(pos!(x, fp.placement.pos.1), &fp.grid().bit_grid) {
                return false;
            }
        }
        true
    }
    pub fn check_rotation_by_srs(&self, fp: &FallingPiece, cw: bool) -> Option<Placement> {
        let after: Orientation = fp.placement.orientation.rotate(if cw { 1 } else { -1 });
        let spec = PieceSpec::of(fp.piece);
        let next_grid: &HybridGrid = &spec.grids[after.id() as usize];
        let offsets1: &Vec<(PosX, PosY)> = &spec.srs_offset_data[fp.placement.orientation.id() as usize];
        let offsets2: &Vec<(PosX, PosY)> = &spec.srs_offset_data[after.id() as usize];
        for i in 0..offsets1.len() {
            let p = fp.placement.pos + Pos::from(offsets1[i]) - Pos::from(offsets2[i]);
            if self.grid.bit_grid.can_put_fast(p, &next_grid.bit_grid) {
                return Some(Placement::new(after, p));
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
                let pos = pos!(fp.placement.pos.0 + dx, fp.placement.pos.1 + dy);
                let is_wall = pos.0 < 0 || pos.1 < 0;
                if is_wall || self.grid.has_cell(pos.into()) {
                    num_corners += 1;
                    if match fp.placement.orientation {
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
                } else if let Some((p1, p2)) = fp.last_two_placements() {
                    let is_shifted = p1.pos.0 != p2.pos.0;
                    let num_rows = p1.pos.0 - p2.pos.0;
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
        tmp_grid.put_fast(fp.placement.pos, &fp.grid().bit_grid);
        LineClear::new(tmp_grid.num_filled_rows(), self.check_tspin(fp, mode))
    }
    pub fn lock(&mut self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<LineClear> {
        if !self.can_lock(fp) {
            return None;
        }
        let tspin = self.check_tspin(fp, mode);
        self.grid.put_fast(fp.placement.pos, &fp.grid().bit_grid);
        let num_cleared_line = self.grid.drop_filled_rows();
        Some(LineClear::new(num_cleared_line, tspin))
    }
    // The return placements can include unreachable placements.
    pub fn search_lockable_placements(&self, piece: Piece) -> Vec<Placement> {
        let yend = (self.grid.height() - self.grid.top_padding()) as PosY;
        let spec = PieceSpec::of(piece);
        let sub_bit_grids = [
            &spec.grids[ORIENTATION_0.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_1.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_2.id() as usize].bit_grid,
            &spec.grids[ORIENTATION_3.id() as usize].bit_grid,
        ];
        let mut r: Vec<Placement> = Vec::new();
        for y in -1..yend {
            for x in -1..(self.grid.width() as PosX - 1) {
                for o in &ORIENTATIONS {
                    let g = sub_bit_grids[o.id() as usize];
                    let can_put = self.grid.bit_grid.can_put_fast((x, y).into(), g);
                    if !can_put {
                        continue;
                    }
                    let can_drop = self.grid.bit_grid.can_put_fast((x, y - 1).into(), g);
                    if can_drop {
                        continue;
                    }
                    r.push(Placement::new(*o, (x, y).into()));
                }
            }
        }
        r
    }
}

//---

pub struct MoveSearchConfiguration<'a> {
    pf: &'a Playfield,
    piece: Piece,
    src: Placement,
    mode: RotationMode,
    debug: bool,
}

impl<'a> MoveSearchConfiguration<'a> {
    pub fn new(pf: &'a Playfield, piece: Piece, src: Placement, mode: RotationMode, debug: bool) -> Self {
        Self { pf, piece, src, mode, debug }
    }
}

pub trait MoveSearchDirector {
    fn next(&mut self, conf: &MoveSearchConfiguration, fp: &FallingPiece, depth: usize, num_moved: usize) -> Option<Move>;
}

pub struct MoveSearchResult {
    src: Placement,
    found: HashMap<Placement, MoveRecordItem>,
}

impl MoveSearchResult {
    pub fn contains(&self, dst: &Placement) -> bool { self.found.contains_key(dst) }
    pub fn get(&self, dst: &Placement) -> Option<MoveRecord> {
        let mut placement = *dst;
        let mut items: Vec<MoveRecordItem> = Vec::new();
        while let Some(item) = self.found.get(&placement) {
            items.push(*item);
            placement = item.placement;
        }
        if items.is_empty() {
            return None;
        }
        let mut record = MoveRecord::new(self.src);
        for item in items.iter().rev() {
            record.merge_or_push(*item);
        }
        Some(record)
    }
    fn len(&self) -> usize { self.found.len() }
}

pub fn search_moves(conf: &MoveSearchConfiguration, director: &mut impl MoveSearchDirector) -> MoveSearchResult {
    type Found = HashMap<Placement, MoveRecordItem>;
    let mut found: Found = HashMap::new();

    fn search(conf: &MoveSearchConfiguration, director: &mut impl MoveSearchDirector,
              fp: &FallingPiece, depth: usize, found: &mut Found) {
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

        debug_println!("search_all: {:?} {}", fp.placement.orientation, fp.placement.pos);
        if depth > 0 && fp.placement == conf.src {
            debug_println!("=> initial placement.");
            return;
        }
        if found.contains_key(&fp.placement) {
            debug_println!("=> already checked.");
            return;
        }
        debug_assert!(fp.move_record.len() <= 1);
        if let Some(last) = fp.move_record.last() {
            let from = MoveRecordItem::new(last.by, fp.move_record.initial_placement);
            let v = found.insert(fp.placement, from);
            debug_assert!(v.is_none());
        }

        let mut fp = FallingPiece::new(fp.piece, fp.placement);
        let mut n = 0;
        while let Some(mv) = director.next(conf, &fp, depth, n) {
            debug_println!("├ {:?}", mv);
            if fp.apply_move(mv, conf.pf, conf.mode) {
                search(conf, director, &fp, depth + 1, found);
                fp.rollback();
            }
            n += 1;
        }
        debug_println!("=> checked.");
    };

    search(conf, director, &FallingPiece::new(conf.piece, conf.src), 0, &mut found);

    MoveSearchResult { src: conf.src, found }
}

pub struct BruteForceMoveSearchDirector();

impl MoveSearchDirector for BruteForceMoveSearchDirector {
    fn next(&mut self, _conf: &MoveSearchConfiguration, _fp: &FallingPiece, _depth: usize, num_moved: usize) -> Option<Move> {
        static MOVES: [Move; 5] = [Move::Shift(1), Move::Drop(1), Move::Shift(-1), Move::Rotate(1), Move::Rotate(-1)];
        MOVES.get(num_moved).copied()
    }
}

pub struct HumanlyOptimizedMoveSearchDirector {}

impl MoveSearchDirector for HumanlyOptimizedMoveSearchDirector {
    fn next(&mut self, _conf: &MoveSearchConfiguration, _fp: &FallingPiece, _depth: usize, _num_moved: usize) -> Option<Move> {
        panic!("TODO");
    }
}

pub struct AStarMoveSearchDirector {}

impl MoveSearchDirector for AStarMoveSearchDirector {
    fn next(&mut self, _conf: &MoveSearchConfiguration, _fp: &FallingPiece, _depth: usize, _num_moved: usize) -> Option<Move> {
        panic!("TODO");
    }
}

//---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_grid() {
        let mut grid = BitGrid::new((4, 5).into());
        assert!(!grid.has_cell((3, 1).into()));
        grid.set_cell((3, 1).into(), Cell::Block(Block::Any));
        assert!(grid.has_cell((3, 1).into()));
        assert_eq!("    \n    \n    \n   @\n    ", format!("{}", grid));

        let mut grid2 = BitGrid::new((3, 2).into());
        grid2.set_cell((1, 0).into(), Cell::Block(Block::Any));
        grid2.set_cell((2, 0).into(), Cell::Block(Block::Any));
        assert!(grid.can_put((1, 0).into(), &grid2));
        assert!(grid.can_put_fast((1, 0).into(), &grid2));
        assert!(!grid.can_put((1, 1).into(), &grid2));
        assert!(!grid.can_put_fast((1, 1).into(), &grid2));
        assert!(grid.can_put((1, 2).into(), &grid2));
        assert!(grid.can_put_fast((1, 2).into(), &grid2));
        grid.put_fast((1, 0).into(), &grid2);
        assert!(grid.has_cell((2, 0).into()));
        assert!(grid.has_cell((3, 0).into()));
    }

    #[test]
    fn test_search_moves() {
        let mut pf = Playfield::new();
        pf.set_rows((0, 0).into(), &[
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
        let fp = FallingPiece::spawn(Piece::T, Some(&pf));
        let placement_to_be_found = Placement::new(ORIENTATION_3, (1, 0).into());
        let lockable = pf.search_lockable_placements(fp.piece);
        assert!(lockable.iter().any(|p| { *p == placement_to_be_found }));
        let search_result = search_moves(
            &MoveSearchConfiguration::new(&pf, fp.piece, fp.placement, RotationMode::Srs, false),
            &mut BruteForceMoveSearchDirector(),
        );
        assert!(search_result.get(&placement_to_be_found).is_some());
        {
            let mut moves: Vec<MoveRecord> = Vec::new();
            for p in &lockable {
                if let Some(record) = search_result.get(p) {
                    moves.push(record);
                }
            }
            assert!(search_result.len() > moves.len());
        }
    }
}
