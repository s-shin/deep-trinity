#[macro_use]
extern crate lazy_static;

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct Orientation(u8);

const ORIENTATION_0: Orientation = Orientation(0);
const ORIENTATION_1: Orientation = Orientation(1);
const ORIENTATION_2: Orientation = Orientation(2);
const ORIENTATION_3: Orientation = Orientation(3);

impl Orientation {
    fn new(n: u8) -> Self { Orientation(n % 4) }
    fn normalize(&mut self) {
        self.0 %= 4;
    }
    fn is(&self, n: u8) -> bool {
        debug_assert!(n < 4);
        self.0 % 4 == n
    }
    fn rotate(self, n: i8) -> Self {
        let mut n = (self.0 as i8 + n) % 4;
        if n < 0 {
            n += 4;
        }
        Self(n as u8)
    }
    fn value(self) -> u8 { self.0 % 4 }
}

//---

#[derive(Copy, Clone, Debug)]
enum Move {
    Horizontally(i8),
    Vertically(i8),
    Rotation(i8),
}

impl Move {
    fn merge(self, m2: Move) -> Option<Move> {
        let m1 = self;
        match m2 {
            Move::Horizontally(n2) => {
                if n2 == 0 {
                    return Some(m1);
                }
                if let Move::Horizontally(n1) = m1 {
                    let n = n1 + n2;
                    if n != 0 {
                        return Some(Move::Horizontally(n));
                    }
                }
            }
            Move::Vertically(n2) => {
                if n2 == 0 {
                    return Some(m1);
                }
                if let Move::Vertically(n1) = m1 {
                    let n = n1 + n2;
                    if n != 0 {
                        return Some(Move::Vertically(n));
                    }
                }
            }
            Move::Rotation(n2) => {
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
struct MoveLogItem {
    pub by: Move,
    pub pos: Pos,
}

#[derive(Clone, Debug)]
struct MoveLog {
    pub initial_pos: Pos,
    pub items: Vec<MoveLogItem>,
}

impl MoveLog {
    fn new(initial_pos: Pos) -> Self {
        Self {
            initial_pos,
            items: Vec::new(),
        }
    }
    fn len(&self) -> usize { self.items.len() }
    fn push(&mut self, item: MoveLogItem) { self.items.push(item); }
    fn get(&self, i: usize) -> Option<&MoveLogItem> { self.items.get(i) }
    fn get_mut(&mut self, i: usize) -> Option<&mut MoveLogItem> { self.items.get_mut(i) }
}

// #[derive(Clone, Debug)]
// struct MovePath(Vec<Move>);
//
// impl MovePath {
//     fn join(&mut self, path: &MovePath) {
//         self.0.extend(path.0.clone());
//     }
//     fn push(&mut self, mv: Move) {
//         if self.0.is_empty() {
//             self.0.push(mv);
//             return;
//         }
//         let m1 = self.0.pop().unwrap();
//         let m2 = mv;
//         match m2 {
//             Move::Horizontally(n2) => {
//                 if n2 == 0 {
//                     self.0.push(m1);
//                     return;
//                 }
//                 if let Move::Horizontally(n1) = m1 {
//                     let n = n1 + n2;
//                     if n != 0 {
//                         self.0.push(Move::Horizontally(n));
//                         return;
//                     }
//                 }
//             }
//             Move::Vertically(n2) => {
//                 if n2 == 0 {
//                     self.0.push(m1);
//                     return;
//                 }
//                 if let Move::Vertically(n1) = m1 {
//                     let n = n1 + n2;
//                     if n != 0 {
//                         self.0.push(Move::Vertically(n));
//                         return;
//                     }
//                 }
//             }
//             Move::Rotation(n2) => {
//                 if n2 == 0 {
//                     self.0.push(m1);
//                     return;
//                 }
//             }
//             // NOTE: Rotations cannot be merged.
//         }
//         self.0.push(m1);
//         self.0.push(m2);
//     }
//     fn normalize(&mut self) {
//         let mut mvs: Vec<Move> = Vec::new();
//         mvs.append(&mut self.0);
//         debug_assert!(self.0.is_empty());
//         for mv in mvs {
//             self.push(mv);
//         }
//     }
// }

//---

#[derive(Copy, Clone, Debug, PartialEq)]
enum TSpin {
    Standard,
    Mini,
}

#[derive(Copy, Clone, Debug)]
struct LineClear {
    pub num_lines: u8,
    pub tspin: Option<TSpin>,
}

impl LineClear {
    fn new(num_lines: u8, tspin: Option<TSpin>) -> Self {
        Self { num_lines, tspin }
    }
    fn is_normal(&self) -> bool { self.tspin.is_none() }
    fn is_tetris(&self) -> bool { self.is_normal() && self.num_lines == 4 }
    fn is_tspin(&self) -> bool {
        if let Some(tspin) = self.tspin {
            tspin == TSpin::Standard
        } else {
            false
        }
    }
    fn is_tspin_mini(&self) -> bool {
        if let Some(tspin) = self.tspin {
            tspin == TSpin::Mini
        } else {
            false
        }
    }
}

//---

#[derive(Copy, Clone, Debug, PartialEq)]
enum RotationMode {
    Srs,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum TSpinJudgementMode {
    PuyoPuyoTetris,
}

//---

// 0: Empty
// 1: Any
// 2-8: S, Z, L, J, I, T, O
// 9: Garbage
struct CellType(u8);

#[derive(Copy, Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug)]
struct BasicGrid {
    size: Size,
    cells: Vec<Cell>,
}

impl BasicGrid {
    fn new(size: Size) -> Self {
        Self {
            size,
            cells: vec![Cell::Empty; (size.0 * size.1) as usize],
        }
    }
    fn pos_to_index(&self, pos: UPos) -> usize {
        debug_assert!(pos.0 < self.width());
        debug_assert!(pos.1 < self.height());
        let idx = (pos.0 + pos.1 * self.width()) as usize;
        idx
    }
    fn rotate_cw(&self) -> Self {
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
    fn put_fast(&mut self, pos: Pos, sub: &BitGrid) -> bool {
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
    fn can_put_fast(&self, pos: Pos, sub: &BitGrid) -> bool {
        debug_assert!(self.width() >= sub.width());
        debug_assert!(self.height() >= sub.height());
        let nshift = if pos.0 < 0 { -pos.0 } else { pos.0 } as BitGridRow;
        let to_right = pos.0 >= 0;
        let edge_checker = if to_right {
            1 << (self.width() - 1) as BitGridRow
        } else {
            1
        };
        for sub_y in 0..sub.height() {
            let mut row = sub.rows[sub_y as usize];
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
            let y = pos.1 as usize + sub_y as usize;
            if self.rows[y] & row != 0 {
                return false;
            }
        }
        true
    }
    fn num_droppable_rows_fast(&self, pos: Pos, sub: &BitGrid) -> SizeY {
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
                let y = pos.1 as usize + sub_y as usize;
                if self.rows[y] & rows_cache[sub_y as usize] != 0 {
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

//---

#[derive(Clone, Debug)]
struct HybridGrid {
    pub basic_grid: BasicGrid,
    pub bit_grid: BitGrid,
}

impl HybridGrid {
    fn new(size: Size) -> Self {
        Self {
            basic_grid: BasicGrid::new(size),
            bit_grid: BitGrid::new(size),
        }
    }
    fn with_grids(basic_grid: BasicGrid, bit_grid: BitGrid) -> Self {
        debug_assert_eq!(basic_grid.size(), bit_grid.size());
        Self {
            basic_grid,
            bit_grid,
        }
    }
    fn put_fast(&mut self, pos: Pos, sub: &BitGrid) {
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

struct PieceSpec {
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
    fn of(piece: Piece) -> &'static Self { &PIECE_SPECS[piece as usize] }
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
    static ref PIECE_SPECS: Vec<PieceSpec> = gen_piece_specs();
}

//---

#[derive(Clone, Debug)]
struct FallingPiece {
    pub piece: Piece,
    pub orientation: Orientation,
    pub pos: Pos,
    pub move_log: MoveLog,
}

impl FallingPiece {
    fn new(piece: Piece, orientation: Orientation, pos: Pos) -> Self {
        Self { piece, orientation, pos, move_log: MoveLog::new(pos) }
    }
    fn grid(&self) -> &'static HybridGrid {
        &PieceSpec::of(self.piece).grids[self.orientation.value() as usize]
    }
    fn apply_move(&mut self, mv: Move, pf: &Playfield, mode: RotationMode) -> bool {
        debug_assert_eq!(RotationMode::Srs, mode);
        match mv {
            Move::Horizontally(n) => {
                if !pf.can_move_horizontally(self, n) {
                    return false;
                }
                self.pos.0 += n;
            }
            Move::Vertically(n) => {
                if !pf.can_drop(self, n as SizeY) {
                    return false;
                }
                self.pos.1 += n
            }
            Move::Rotation(n) => {
                let backup = (self.orientation, self.pos);
                for _ in 0..(if n > 0 { n } else { n }) {
                    if let Some(pos) = pf.check_rotation_by_srs(self, n > 0) {
                        self.orientation = self.orientation.rotate(if n > 0 { 1 } else { -1 });
                        self.pos = pos;
                    } else {
                        self.orientation = backup.0;
                        self.pos = backup.1;
                        return false;
                    }
                }
            }
        }
        true
    }
    fn is_last_move_rotation(&self) -> bool {
        if let Some(item) = self.move_log.items.last() {
            if let Move::Rotation(_) = item.by {
                return true;
            }
        }
        false
    }
    fn last_two_positions(&self) -> Option<(Pos, Pos)> {
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

const PLAYFIELD_SIZE: Size = (10, 40);
const PLAYFIELD_VISIBLE_HEIGHT: SizeY = 20;

#[derive(Clone, Debug)]
struct Playfield {
    pub grid: HybridGrid,
}

impl Playfield {
    fn new() -> Self {
        Self {
            grid: HybridGrid::new(PLAYFIELD_SIZE),
        }
    }
    fn can_put(&self, fp: &FallingPiece) -> bool {
        self.grid.bit_grid.can_put(fp.pos, &fp.grid().bit_grid)
    }
    fn num_droppable_rows(&self, fp: &FallingPiece) -> SizeY {
        self.grid.bit_grid.num_droppable_rows_fast(fp.pos, &fp.grid().bit_grid)
    }
    fn can_drop(&self, fp: &FallingPiece, n: SizeY) -> bool {
        self.grid.bit_grid.can_put_fast((fp.pos.0, fp.pos.1 - n as PosY), &fp.grid().bit_grid)
    }
    fn can_move_horizontally(&self, fp: &FallingPiece, n: PosX) -> bool {
        let to_right = n > 0;
        let end = if to_right { n } else { -n };
        for dx in 0..end {
            let x = fp.pos.0 + if to_right { dx } else { -dx };
            if !self.grid.bit_grid.can_put_fast((x, fp.pos.1), &fp.grid().bit_grid) {
                return false;
            }
        }
        true
    }
    fn check_rotation_by_srs(&self, fp: &FallingPiece, cw: bool) -> Option<Pos> {
        let after: Orientation = fp.orientation.rotate(if cw { 1 } else { -1 });
        let spec = PieceSpec::of(fp.piece);
        let next_grid: &HybridGrid = &spec.grids[after.value() as usize];
        let offsets1: &Vec<Pos> = &spec.srs_offset_data[fp.orientation.value() as usize];
        let offsets2: &Vec<Pos> = &spec.srs_offset_data[after.value() as usize];
        for i in 0..offsets1.len() {
            let (mut x, mut y) = fp.pos;
            x += offsets1[i].0 - offsets2[i].0;
            y += offsets1[i].1 - offsets2[i].1;
            if self.grid.bit_grid.can_put_fast((x, y), &next_grid.bit_grid) {
                return Some((x, y));
            }
        }
        None
    }
    fn can_lock(&self, fp: &FallingPiece) -> bool { self.can_put(fp) && !self.can_drop(fp, 1) }
    fn check_tspin(&self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<TSpin> {
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
                let pos = (fp.pos.0 + dx, fp.pos.1 + dy);
                let is_wall = pos.0 < 0 || pos.1 < 0;
                if is_wall || self.grid.has_cell((pos.0 as UPosX, pos.1 as UPosY)) {
                    num_corners += 1;
                    if match fp.orientation {
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
    fn check_line_clear(&self, fp: &FallingPiece, mode: TSpinJudgementMode) -> LineClear {
        debug_assert!(self.can_lock(fp));
        let mut tmp_grid = self.grid.clone();
        tmp_grid.put_fast(fp.pos, &fp.grid().bit_grid);
        LineClear::new(tmp_grid.num_filled_rows(), self.check_tspin(fp, mode))
    }
    fn lock(&mut self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<LineClear> {
        if !self.can_lock(fp) {
            return None;
        }
        let tspin = self.check_tspin(fp, mode);
        self.grid.put_fast(fp.pos, &fp.grid().bit_grid);
        let num_cleared_line = self.grid.drop_filled_rows();
        Some(LineClear::new(num_cleared_line, tspin))
    }
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
