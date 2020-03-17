#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate lazy_static;
extern crate rand;

use std::collections::{HashMap, VecDeque, BTreeMap};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops;
use rand::seq::SliceRandom;

//---

pub type PosX = i8;
pub type PosY = i8;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Pos(pub PosX, pub PosY);

#[macro_export]
macro_rules! pos {
    ($x:expr, $y:expr) => { $crate::Pos($x, $y) }
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
pub struct UPos(pub SizeX, pub SizeY);

#[macro_export]
macro_rules! upos {
    ($x:expr, $y:expr) => { $crate::UPos($x, $y) }
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

#[macro_export]
macro_rules! size {
    ($x:expr, $y:expr) => { $crate::UPos($x, $y) }
}

/// Pos -> UPos
impl From<Pos> for UPos {
    fn from(pos: Pos) -> Self {
        debug_assert!(pos.0 >= 0);
        debug_assert!(pos.1 >= 0);
        Self(pos.0 as SizeX, pos.1 as SizeY)
    }
}

/// UPos -> Pos
impl From<UPos> for Pos {
    fn from(pos: UPos) -> Self {
        debug_assert!(pos.0 <= PosX::max_value() as UPosX);
        debug_assert!(pos.1 <= PosY::max_value() as UPosY);
        Self(pos.0 as PosX, pos.1 as PosY)
    }
}

/// UPos + Pos -> UPos
impl ops::Add<Pos> for UPos {
    type Output = Self;
    fn add(self, other: Pos) -> Self { Self::from(Pos::from(self) + other) }
}

/// Pos + UPos -> Pos
impl ops::Add<UPos> for Pos {
    type Output = Self;
    fn add(self, other: UPos) -> Self { self + Self::from(other) }
}

//---

pub trait Grid: Clone + fmt::Display {
    fn size(&self) -> Size;
    fn width(&self) -> SizeX { self.size().0 }
    fn height(&self) -> SizeY { self.size().1 }
    fn get_cell(&self, pos: UPos) -> Cell;
    fn set_cell(&mut self, pos: UPos, cell: Cell);
    fn has_cell(&self, pos: UPos) -> bool { !self.get_cell(pos).is_empty() }
    fn is_empty(&self) -> bool { self.bottom_padding() == self.height() }
    fn is_valid_pos(&self, pos: Pos) -> bool {
        0 <= pos.0 && pos.0 < self.width() as PosX && 0 <= pos.1 && pos.1 < self.height() as PosY
    }
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
                if !self.is_valid_pos(p) {
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
                if !self.is_valid_pos(p) {
                    return false;
                }
                if self.has_cell(p.into()) {
                    return false;
                }
            }
        }
        true
    }
    fn get_last_pos<G: Grid>(&self, pos: Pos, sub: &G, delta: Pos) -> Pos {
        let mut p = pos;
        loop {
            let pp = p + delta;
            if !self.can_put(pp, sub) {
                return p;
            }
            p = pp;
        }
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
    fn set_cell_to_row(&mut self, y: UPosY, cell: Cell) {
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
                self.set_cell_to_row(y, Cell::Empty);
                n += 1
            } else if n > 0 {
                self.swap_rows(y - n, y);
            }
        }
        n
    }
    /// `false` will be returned if any non-empty cells are disposed.
    fn insert_cell_to_rows(&mut self, y: UPosY, cell: Cell, n: SizeY, force: bool) -> bool {
        debug_assert!(self.height() >= y + n);
        let mut are_cells_disposed = false;
        for y in (self.height() - n)..self.height() {
            if !self.is_row_empty(y) {
                if !force {
                    return false;
                }
                are_cells_disposed = true;
                self.set_cell_to_row(y, cell);
            }
        }
        for y in (0..(self.height() - n)).rev() {
            self.swap_rows(y, y + n);
        }
        !are_cells_disposed
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
            let y = pos.1 + sub_y as PosY;
            if y < 0 || self.height() as PosY <= y {
                if row != 0 {
                    dirty = true;
                }
                continue;
            }
            let y = y as usize;
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
    fn set_cell_to_row(&mut self, y: UPosY, cell: Cell) {
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
    pub fn put_fast(&mut self, pos: Pos, sub: &HybridGrid) {
        self.basic_grid.put(pos, &sub.basic_grid);
        self.bit_grid.put_fast(pos, &sub.bit_grid);
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
    fn set_cell_to_row(&mut self, y: UPosY, cell: Cell) {
        self.basic_grid.set_cell_to_row(y, cell);
        self.bit_grid.set_cell_to_row(y, cell);
    }
}

impl fmt::Display for HybridGrid {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

impl PartialEq for HybridGrid {
    fn eq(&self, other: &Self) -> bool { self.basic_grid == other.basic_grid }
}

impl Hash for HybridGrid {
    fn hash<H: Hasher>(&self, state: &mut H) { self.basic_grid.hash(state); }
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
    pub orientation: Orientation,
    pub pos: Pos,
}

impl Placement {
    fn new(orientation: Orientation, pos: Pos) -> Self {
        Self { orientation, pos }
    }
}

//---

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MoveRecordItem {
    pub by: Move,
    pub placement: Placement,
}

impl MoveRecordItem {
    fn new(by: Move, placement: Placement) -> Self {
        Self { by, placement }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum TSpin {
    Standard,
    Mini,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

impl fmt::Display for LineClear {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let n = self.num_lines as usize;
        if self.is_normal() {
            static STRS: [&'static str; 5] = ["zero", "single", "double", "triple", "tetris"];
            if n < STRS.len() {
                write!(f, "{}", STRS[n])?;
            } else {
                write!(f, "{}", n)?;
            }
        } else if self.is_tspin() {
            static STRS: [&'static str; 4] = ["tsz", "tss", "tsd", "tst"];
            if n < STRS.len() {
                write!(f, "{}", STRS[n])?;
            } else {
                write!(f, "ts{}", n)?;
            }
        } else if self.is_tspin_mini() {
            static STRS: [&'static str; 3] = ["tsmz", "tsms", "tsmd"];
            if n < STRS.len() {
                write!(f, "{}", STRS[n])?;
            } else {
                write!(f, "tsm{}", n)?;
            }
        }
        Ok(())
    }
}

//---

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum RotationMode {
    Srs,
}

impl Default for RotationMode {
    fn default() -> Self { Self::Srs }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TSpinJudgementMode {
    PuyoPuyoTetris,
}

impl Default for TSpinJudgementMode {
    fn default() -> Self { Self::PuyoPuyoTetris }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LockOutType {
    LockOut,
    PartialLockOut,
}

bitflags! {
    pub struct LossConditions: u8 {
        const BLOCK_OUT        = 0b0001;
        const LOCK_OUT         = 0b0110;  // LOCK_OUT includes PARTIAL_LOCK_OUT.
        const PARTIAL_LOCK_OUT = 0b0100;
        const GARBAGE_OUT      = 0b1000;
    }
}

impl Default for LossConditions {
    fn default() -> Self {
        Self::BLOCK_OUT | Self::LOCK_OUT | Self::GARBAGE_OUT
    }
}

//---

/// 0: Empty
/// 1: Any
/// 2-8: S, Z, L, J, I, T, O
/// 9: Garbage
pub struct CellTypeId(pub u8);

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

const PIECES: [Piece; 7] = [Piece::S, Piece::Z, Piece::L, Piece::J, Piece::I, Piece::T, Piece::O];

impl From<usize> for Piece {
    fn from(n: usize) -> Self {
        assert!(n < 7);
        PIECES[n]
    }
}

impl Into<CellTypeId> for Piece {
    fn into(self) -> CellTypeId { CellTypeId(2 + (self as u8)) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Any,
    Piece(Piece),
    Garbage,
}

impl Into<CellTypeId> for Block {
    fn into(self) -> CellTypeId {
        match self {
            Block::Any => CellTypeId(1),
            Block::Piece(p) => p.into(),
            Block::Garbage => CellTypeId(9),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    Empty,
    Block(Block),
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
        let id: CellTypeId = self.into();
        CELL_CHARS.chars().nth(id.0 as usize).unwrap()
    }
}

impl From<Cell> for CellTypeId {
    fn from(c: Cell) -> Self {
        match c {
            Cell::Empty => Self(0),
            Cell::Block(b) => b.into(),
        }
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
    /// The index of Vec is orientation.
    grids: Vec<HybridGrid>,
    initial_placement: Placement,
    /// The index of outer Vec is orientation.
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
            (2, 17),
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
            Piece::O,
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Playfield {
    pub grid: HybridGrid,
    pub visible_height: SizeY,
}

impl Playfield {
    pub fn new(size: Size, visible_height: SizeY) -> Self {
        Self {
            grid: HybridGrid::new(size),
            visible_height,
        }
    }
    pub fn width(&self) -> SizeX { self.grid.width() }
    pub fn height(&self) -> SizeX { self.grid.height() }
    pub fn is_empty(&self) -> bool { self.grid.is_empty() }
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
    // If garbage out, `true` will be returned.
    pub fn append_garbage(&mut self, gap_x_list: &[UPosX]) -> bool {
        let ok = self.grid.insert_cell_to_rows(0, Cell::Block(Block::Garbage),
                                               gap_x_list.len() as SizeY, true);
        for (y, x) in gap_x_list.iter().enumerate() {
            self.grid.set_cell((*x, y as UPosY).into(), Cell::Empty);
        }
        !ok
    }
    pub fn can_put(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put(fp.placement.pos, &fp.grid().bit_grid)
    }
    pub fn num_droppable_rows(&self, fp: &FallingPiece) -> SizeY {
        self.grid.bit_grid.num_droppable_rows_fast(fp.placement.pos, &fp.grid().bit_grid)
    }
    pub fn num_shiftable_cols(&self, fp: &FallingPiece, to_right: bool) -> SizeX {
        let p = self.grid.bit_grid.get_last_pos(fp.placement.pos, &fp.grid().bit_grid,
                                                if to_right { (1, 0) } else { (-1, 0) }.into());
        let r = if to_right { p.0 - fp.placement.pos.0 } else { fp.placement.pos.0 - p.0 };
        debug_assert!(r >= 0);
        r as SizeX
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
        for dx in 1..=end {
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
    // This method doesn't consider whether the game is over or not.
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
        tmp_grid.put_fast(fp.placement.pos, fp.grid());
        LineClear::new(tmp_grid.num_filled_rows(), self.check_tspin(fp, mode))
    }
    pub fn check_lock_out(&self, fp: &FallingPiece) -> Option<LockOutType> {
        let bottom = fp.placement.pos.1 + fp.grid().bottom_padding() as PosY;
        if bottom >= self.visible_height as PosY {
            return Some(LockOutType::LockOut);
        }
        let top = fp.placement.pos.1 + fp.grid().height() as PosY - fp.grid().top_padding() as PosY;
        if top >= self.visible_height as PosY {
            return Some(LockOutType::PartialLockOut);
        }
        None
    }
    pub fn lock(&mut self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<LineClear> {
        if !self.can_lock(fp) {
            return None;
        }
        let tspin = self.check_tspin(fp, mode);
        self.grid.put_fast(fp.placement.pos, fp.grid());
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

pub const DEFAULT_PLAYFIELD_SIZE: Size = size!(10, 40);
pub const DEFAULT_PLAYFIELD_VISIBLE_HEIGHT: SizeY = 20;

impl Default for Playfield {
    fn default() -> Self {
        Self::new(DEFAULT_PLAYFIELD_SIZE, DEFAULT_PLAYFIELD_VISIBLE_HEIGHT)
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
    pub fn len(&self) -> usize { self.found.len() }
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

// TODO
// pub struct HumanlyOptimizedMoveSearchDirector {}
//
// impl MoveSearchDirector for HumanlyOptimizedMoveSearchDirector {
//     fn next(&mut self, _conf: &MoveSearchConfiguration, _fp: &FallingPiece, _depth: usize, _num_moved: usize) -> Option<Move> {
//         panic!("TODO");
//     }
// }

// TODO
// pub struct AStarMoveSearchDirector {}
//
// impl MoveSearchDirector for AStarMoveSearchDirector {
//     fn next(&mut self, _conf: &MoveSearchConfiguration, _fp: &FallingPiece, _depth: usize, _num_moved: usize) -> Option<Move> {
//         panic!("TODO");
//     }
// }

//---

pub const DEFAULT_NUM_VISIBLE_NEXT_PIECES: usize = 5;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct NextPieces {
    pub pieces: VecDeque<Piece>,
    pub visible_num: usize,
}

impl NextPieces {
    pub fn new(visible_num: usize) -> Self { Self { pieces: VecDeque::new(), visible_num } }
    pub fn is_empty(&self) -> bool { self.pieces.is_empty() }
    pub fn len(&self) -> usize { self.pieces.len() }
    pub fn iter(&self) -> std::collections::vec_deque::Iter<Piece> { self.pieces.iter() }
    pub fn pop(&mut self) -> Option<Piece> { self.pieces.pop_front() }
    pub fn supply(&mut self, ps: &[Piece]) { self.pieces.extend(ps) }
    pub fn should_supply(&self) -> bool { self.len() <= self.visible_num }
}

impl Default for NextPieces {
    fn default() -> Self { Self::new(DEFAULT_NUM_VISIBLE_NEXT_PIECES) }
}

impl fmt::Display for NextPieces {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (i, p) in self.iter().enumerate() {
            if i >= self.visible_num {
                break;
            }
            write!(f, "{}", Cell::Block(Block::Piece(*p)).char())?;
        }
        Ok(())
    }
}

//---

pub type Count = u32;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LineClearCounter {
    pub data: HashMap<LineClear, Count>,
}

impl LineClearCounter {
    pub fn add(&mut self, lc: &LineClear, n: Count) {
        if let Some(c) = self.data.get_mut(lc) {
            *c += n;
        } else {
            self.data.insert(*lc, n);
        }
    }
    pub fn get(&self, lc: &LineClear) -> Count {
        self.data.get(lc).copied().unwrap_or(0)
    }
}

impl ops::Sub for LineClearCounter {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let mut r = Self::default();
        for (lc, count) in self.data.iter() {
            r.add(lc, *count - other.get(lc));
        }
        r
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ConsecutiveCountCounter {
    pub data: BTreeMap<Count, Count>,
}

impl ConsecutiveCountCounter {
    pub fn add(&mut self, cont_count: Count, n: Count) {
        if let Some(c) = self.data.get_mut(&cont_count) {
            *c += n;
        } else {
            self.data.insert(cont_count, n);
        }
    }
    pub fn get(&self, cont_count: Count) -> Count {
        self.data.get(&cont_count).copied().unwrap_or(0)
    }
    pub fn max(&self) -> Count {
        self.data.iter().next_back().map_or(0, |v| { *v.0 })
    }
}

impl ops::Sub for ConsecutiveCountCounter {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let mut r = Self::default();
        for (cont_count, count) in self.data.iter() {
            r.add(*cont_count, *count - other.get(*cont_count));
        }
        r
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StatisticsEntryType {
    LineClear(LineClear),
    Combo(Count),
    MaxCombo,
    Btb(Count),
    MaxBtb,
    PerfectClear,
    Hold,
    Lock,
}

impl fmt::Display for StatisticsEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatisticsEntryType::LineClear(lc) => write!(f, "{}", lc),
            StatisticsEntryType::Combo(n) => write!(f, "combo[{}]", n),
            StatisticsEntryType::MaxCombo => write!(f, "max combo"),
            StatisticsEntryType::Btb(n) => write!(f, "btb[{}]", n),
            StatisticsEntryType::MaxBtb => write!(f, "max btb"),
            StatisticsEntryType::PerfectClear => write!(f, "pc"),
            StatisticsEntryType::Hold => write!(f, "hold"),
            StatisticsEntryType::Lock => write!(f, "lock"),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Statistics {
    pub line_clear: LineClearCounter,
    pub combo: ConsecutiveCountCounter,
    pub btb: ConsecutiveCountCounter,
    pub perfect_clear: Count,
    pub hold: Count,
    pub lock: Count,
}

impl Statistics {
    fn get(&self, t: StatisticsEntryType) -> Count {
        match t {
            StatisticsEntryType::LineClear(lc) => self.line_clear.get(&lc),
            StatisticsEntryType::Combo(n) => self.combo.get(n),
            StatisticsEntryType::MaxCombo => self.combo.max(),
            StatisticsEntryType::Btb(n) => self.btb.get(n),
            StatisticsEntryType::MaxBtb => self.btb.max(),
            StatisticsEntryType::PerfectClear => self.perfect_clear,
            StatisticsEntryType::Hold => self.hold,
            StatisticsEntryType::Lock => self.lock,
        }
    }
}

impl ops::Sub for Statistics {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            line_clear: self.line_clear - other.line_clear,
            combo: self.combo - other.combo,
            btb: self.btb - other.btb,
            perfect_clear: self.perfect_clear - other.perfect_clear,
            hold: self.hold - other.hold,
            lock: self.lock - other.lock,
        }
    }
}

//---

pub trait PieceGenerator {
    fn generate(&mut self) -> Vec<Piece>;
}

#[derive(Clone, Debug)]
pub struct RandomPieceGenerator<R: rand::Rng + ?Sized = rand::rngs::StdRng> {
    rng: R,
}

impl<R: rand::Rng + Sized> RandomPieceGenerator<R> {
    pub fn new(rng: R) -> Self { Self { rng } }
}

impl<R: rand::Rng + Sized> PieceGenerator for RandomPieceGenerator<R> {
    fn generate(&mut self) -> Vec<Piece> {
        let mut ps = PIECES.clone();
        ps.shuffle(&mut self.rng);
        ps.to_vec()
    }
}

#[derive(Clone, Debug)]
pub struct StaticPieceGenerator {
    pieces: VecDeque<Piece>,
    num_per_gen: usize,
}

impl StaticPieceGenerator {
    pub fn len(&self) -> usize { self.pieces.len() }
    pub fn append(&mut self, pieces: &[Piece]) { self.pieces.extend(pieces) }
}

impl Default for StaticPieceGenerator {
    fn default() -> Self {
        Self {
            pieces: VecDeque::new(),
            num_per_gen: 7,
        }
    }
}

impl PieceGenerator for StaticPieceGenerator {
    fn generate(&mut self) -> Vec<Piece> {
        let n = std::cmp::min(self.len(), self.num_per_gen);
        self.pieces.drain(0..n).collect()
    }
}

//---

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameState {
    pub playfield: Playfield,
    pub next_pieces: NextPieces,
    pub falling_piece: Option<FallingPiece>,
    pub hold_piece: Option<Piece>,
    pub can_hold: bool,
    pub num_combos: Option<Count>,
    pub num_btbs: Option<Count>,
    pub game_over_reason: LossConditions,
}

impl GameState {
    pub fn is_game_over(&self) -> bool { !self.game_over_reason.is_empty() }
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            playfield: Playfield::default(),
            next_pieces: NextPieces::default(),
            falling_piece: None,
            hold_piece: None,
            can_hold: true,
            num_combos: None,
            num_btbs: None,
            game_over_reason: LossConditions::empty(),
        }
    }
}

//---

/// The standard implementation of game management.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Game<PG: PieceGenerator> {
    pub state: GameState,
    pub stats: Statistics,
    pub piece_gen: PG,
    pub rotation_mode: RotationMode,
    pub tspin_judgement_mode: TSpinJudgementMode,
    pub loss_conds: LossConditions,
}

impl<PG: PieceGenerator> Game<PG> {
    pub fn new(piece_gen: PG) -> Self {
        Self {
            state: Default::default(),
            stats: Default::default(),
            piece_gen,
            rotation_mode: Default::default(),
            tspin_judgement_mode: Default::default(),
            loss_conds: Default::default(),
        }
    }
    pub fn get_cell(&self, pos: UPos) -> Cell {
        let s = &self.state;
        let mut cell = if let Some(fp) = s.falling_piece.as_ref() {
            let grid = fp.grid();
            let grid_pos = Pos::from(pos) - fp.placement.pos;
            if grid.is_valid_pos(grid_pos) {
                grid.get_cell(grid_pos.into())
            } else {
                Cell::Empty
            }
        } else {
            Cell::Empty
        };
        if cell == Cell::Empty {
            cell = s.playfield.grid.get_cell(pos.into());
        }
        cell
    }
    /// This method should be called right after `new()`.
    /// `true` will be returned when there are no next pieces.
    pub fn setup_falling_piece(&mut self, next: Option<Piece>) -> Result<(), &'static str> {
        let s = &mut self.state;

        if s.falling_piece.is_some() {
            return Err("falling piece already exists");
        }

        let p = if let Some(next) = next {
            next
        } else {
            if s.next_pieces.should_supply() {
                s.next_pieces.supply(&self.piece_gen.generate());
            }
            if s.next_pieces.is_empty() {
                return Err("no next pieces");
            }
            s.next_pieces.pop().unwrap()
        };

        let fp = FallingPiece::spawn(p, Some(&s.playfield));
        if !s.playfield.can_put(&fp) {
            s.game_over_reason |= LossConditions::BLOCK_OUT;
        }
        s.falling_piece = Some(fp);
        s.can_hold = true;
        Ok(())
    }
    /// `Err` will be returned when an invalid move was specified.
    pub fn do_move(&mut self, mv: Move) -> Result<(), &'static str> {
        if self.state.falling_piece.is_none() {
            return Err("no falling piece");
        }
        let fp = self.state.falling_piece.as_mut().unwrap();
        if fp.apply_move(mv, &self.state.playfield, self.rotation_mode) {
            Ok(())
        } else {
            Err("invalid move specified")
        }
    }
    pub fn drop(&mut self, n: i8) -> Result<(), &'static str> {
        self.drop_internal(n, false)
    }
    pub fn firm_drop(&mut self) -> Result<(), &'static str> {
        self.drop_internal(0, true)
    }
    fn drop_internal(&mut self, n: i8, is_firm: bool) -> Result<(), &'static str> {
        let s = &mut self.state;
        if s.falling_piece.is_none() {
            return Err("no falling piece");
        }
        let n = if is_firm {
            s.playfield.num_droppable_rows(s.falling_piece.as_ref().unwrap()) as i8
        } else {
            n
        };
        if n == 0 {
            return Ok(());
        }
        self.do_move(Move::Drop(n))
    }
    pub fn shift(&mut self, n: i8, to_end: bool) -> Result<(), &'static str> {
        let s = &mut self.state;
        if s.falling_piece.is_none() {
            return Err("no falling piece");
        }
        if n == 0 {
            return Ok(());
        }
        let to_right = n > 0;
        let n = if to_end {
            let n = s.playfield.num_shiftable_cols(s.falling_piece.as_ref().unwrap(), to_right) as i8;
            if to_right { n } else { -n }
        } else {
            n
        };
        self.do_move(Move::Shift(n))
    }
    pub fn rotate(&mut self, n: i8) -> Result<(), &'static str> {
        self.do_move(Move::Rotate(n))
    }
    /// `Ok(true)` will be returned if the process is totally succeeded.
    /// If `Ok(false)` was returned, you should supply next pieces then call `setup_next_piece()`.
    /// `Err` will be returned when the process fails.
    pub fn lock(&mut self) -> Result<bool, &'static str> {
        let s = &mut self.state;
        if s.falling_piece.is_none() {
            return Err("falling_piece is none");
        }
        let fp = s.falling_piece.as_mut().unwrap();
        let pf = &mut s.playfield;
        if !pf.can_lock(fp) {
            return Err("cannot lock");
        }
        if let Some(lock_out_type) = pf.check_lock_out(fp) {
            match lock_out_type {
                LockOutType::LockOut => {
                    if self.loss_conds.contains(LossConditions::LOCK_OUT) {
                        s.game_over_reason |= LossConditions::LOCK_OUT;
                    }
                }
                LockOutType::PartialLockOut => {
                    if self.loss_conds.contains(LossConditions::PARTIAL_LOCK_OUT) {
                        s.game_over_reason |= LossConditions::PARTIAL_LOCK_OUT;
                    }
                }
            }
        }
        let line_clear = pf.lock(fp, self.tspin_judgement_mode);
        s.falling_piece = None;
        debug_assert!(line_clear.is_some());
        let line_clear = line_clear.unwrap();
        self.stats.lock += 1;
        self.stats.line_clear.add(&line_clear, 1);
        if line_clear.num_lines > 0 {
            s.num_combos = Some(s.num_combos.map_or(0, |n| { n + 1 }));
            self.stats.combo.add(s.num_combos.unwrap(), 1);
            if pf.is_empty() {
                self.stats.perfect_clear += 1;
            }
            if line_clear.is_tetris() || line_clear.is_tspin() || line_clear.is_tspin_mini() {
                s.num_btbs = Some(s.num_btbs.map_or(0, |n| { n + 1 }));
                self.stats.btb.add(s.num_btbs.unwrap(), 1);
            } else {
                s.num_btbs = None;
            }
        } else {
            s.num_btbs = None;
            s.num_combos = None;
        }
        Ok(self.setup_falling_piece(None).is_ok())
    }
    /// `Ok(true)` will be returned if the process is totally succeeded.
    /// If `Ok(false)` was returned, you should supply next pieces then call `setup_next_piece()`.
    /// `Err` will be returned when the process fails.
    pub fn hold(&mut self) -> Result<bool, &'static str> {
        let s = &mut self.state;
        if s.falling_piece.is_none() {
            return Err("no falling piece");
        }
        if !s.can_hold {
            return Err("already held once");
        }
        let p = s.falling_piece.as_ref().unwrap().piece;
        s.falling_piece = None;
        let r = self.setup_falling_piece(self.state.hold_piece);
        self.state.hold_piece = Some(p);
        self.state.can_hold = false;
        self.stats.hold += 1;
        Ok(r.is_ok())
    }
}

impl<PG: PieceGenerator> fmt::Display for Game<PG> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = &self.state;
        let w = self.state.playfield.width() as usize;
        let h = self.state.playfield.visible_height as usize;
        let num_next = self.state.next_pieces.visible_num;
        write!(f, "[{}]", s.hold_piece.map_or(
            Cell::Empty, |p| { Cell::Block(Block::Piece(p)) }).char(),
        )?;
        write!(f, "{}", " ".repeat(w - num_next - 3))?;
        write!(f, "({})", s.falling_piece.as_ref().map_or(
            Cell::Empty, |fp| { Cell::Block(Block::Piece(fp.piece)) }).char(),
        )?;
        writeln!(f, "{}", s.next_pieces)?;
        writeln!(f, "--+{}+", "-".repeat(w))?;
        for i in 0..h {
            let y = h - 1 - i;
            write!(f, "{:02}|", y)?;
            for x in 0..w {
                let cell = self.get_cell(upos!(x as UPosX, y as UPosY));
                write!(f, "{}", cell.char())?;
            }
            write!(f, "|")?;
            match i {
                0 | 1 | 2 | 3 => {
                    let t = StatisticsEntryType::LineClear(LineClear::new(i as u8 + 1, None));
                    write!(f, "  {:6}  {}", format!("{}", t).to_ascii_uppercase(), self.stats.get(t))?;
                }
                4 | 5 | 6 => {
                    let t = StatisticsEntryType::LineClear(LineClear::new(7 - i as u8, Some(TSpin::Standard)));
                    write!(f, "  {:6}  {}", format!("{}", t).to_ascii_uppercase(), self.stats.get(t))?;
                }
                7 | 8 => {
                    let t = StatisticsEntryType::LineClear(LineClear::new(9 - i as u8, Some(TSpin::Mini)));
                    write!(f, "  {:6}  {}", format!("{}", t).to_ascii_uppercase(), self.stats.get(t))?;
                }
                9 => {
                    write!(f, "  {:6}  {}/{}", "COMBO", s.num_combos.unwrap_or(0), self.stats.get(StatisticsEntryType::MaxCombo))?;
                }
                10 => {
                    write!(f, "  {:6}  {}/{}", "BTB", s.num_combos.unwrap_or(0), self.stats.get(StatisticsEntryType::MaxBtb))?;
                }
                11 => {
                    write!(f, "  {:6}  {}", "HOLD", self.stats.get(StatisticsEntryType::Hold))?;
                }
                12 => {
                    write!(f, "  {:6}  {}", "LOCK", self.stats.get(StatisticsEntryType::Lock))?;
                }
                _ => {}
            }
            writeln!(f)?;
        }
        writeln!(f, "--+{}+", "-".repeat(w))?;
        write!(f, "##|")?;
        for x in 0..w {
            write!(f, "{}", x % 10)?;
        }
        write!(f, "|")?;
        Ok(())
    }
}

//---

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_ok {
        ($r:expr) => {
            match $r {
                Ok(v) => v,
                Err(e) => panic!("assertion failed: Err({:?})", e),
            }
        }
    }

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
        let mut pf = Playfield::default();
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

    #[test]
    fn test_game() {
        let mut game = Game::new(StaticPieceGenerator::default());
        game.piece_gen.append(&[
            Piece::O, Piece::T, Piece::I, Piece::J, Piece::L, Piece::S, Piece::Z,
            Piece::O, Piece::T, Piece::I, Piece::J, Piece::L, Piece::S, Piece::Z,
        ]);
        assert_ok!(game.setup_falling_piece(None));
        // Test simple TSD opener.
        // O
        assert_eq!(Piece::O, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.shift(-1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // T
        assert_eq!(Piece::T, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.hold());
        // I
        assert_eq!(Piece::I, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // J
        assert_eq!(Piece::J, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.rotate(-1));
        assert_ok!(game.shift(1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // L
        assert_eq!(Piece::L, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.rotate(1));
        assert_ok!(game.shift(-1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // S
        assert_eq!(Piece::S, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.shift(1, false));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // Z
        assert_eq!(Piece::Z, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.shift(-2, false));
        assert_ok!(game.rotate(1));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // O
        assert_eq!(Piece::O, game.state.falling_piece.as_ref().unwrap().piece);
        assert_ok!(game.hold());
        // T
        assert_ok!(game.shift(1, true));
        assert_ok!(game.rotate(-1));
        assert_ok!(game.firm_drop());
        assert_ok!(game.rotate(-1));
        assert_ok!(game.lock());

        assert_eq!(r#"[O]  (T)IJLSZ
--+----------+
19|   TTT    |  SINGLE  0
18|          |  DOUBLE  0
17|          |  TRIPLE  0
16|          |  TETRIS  0
15|          |  TST     0
14|          |  TSD     1
13|          |  TSS     0
12|          |  TSMD    0
11|          |  TSMS    0
10|          |  COMBO   0/0
09|          |  BTB     0/0
08|          |  HOLD    2
07|          |  LOCK    7
06|          |
05|          |
04|          |
03|          |
02|L         |
01|L         |
00|LL Z SS  J|
--+----------+
##|0123456789|"#, format!("{}", game));
    }
}
