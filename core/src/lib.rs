pub mod move_search;
pub mod helper;
pub mod prelude;

use std::collections::{HashMap, VecDeque, BTreeMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::ops;
use std::rc::Rc;
use rand::seq::SliceRandom;
use bitflags::bitflags;
use once_cell::sync::Lazy;
use grid::{CellTrait, Grid, X, Y, Vec2};
use grid::bitgrid::BitGridTrait;
use crate::helper::{MoveDecisionHelper, MoveDecisionStuff};

//--------------------------------------------------------------------------------------------------
// Global Configurations
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct GlobalDefaults {
    playfield_size: Vec2,
    playfield_visible_height: Y,
    num_visible_next_pieces: usize,
}

impl GlobalDefaults {
    pub fn new(playfield_size: Vec2, playfield_visible_height: Y, num_visible_next_pieces: usize) -> Self {
        Self { playfield_size, playfield_visible_height, num_visible_next_pieces }
    }
}

impl Default for GlobalDefaults {
    fn default() -> Self {
        Self::new((10, 40).into(), 20, 5)
    }
}

mod global_defaults_internal {
    use once_cell::sync::OnceCell;
    use super::GlobalDefaults;

    static GLOBAL_DEFAULTS: OnceCell<GlobalDefaults> = OnceCell::new();

    pub fn init_global_defaults(v: GlobalDefaults) -> Result<(), &'static str> {
        GLOBAL_DEFAULTS.set(v).map_err(|_| "Already initialized.")
    }

    pub fn global_defaults() -> &'static GlobalDefaults {
        if let Some(c) = GLOBAL_DEFAULTS.get() {
            return c;
        }
        GLOBAL_DEFAULTS.set(Default::default()).ok();
        GLOBAL_DEFAULTS.get().unwrap()
    }
}

pub use global_defaults_internal::init_global_defaults;
use global_defaults_internal::global_defaults;

//--------------------------------------------------------------------------------------------------
// Piece, Block and Cell
//--------------------------------------------------------------------------------------------------

pub const CELL_CHARS: &'static str = " @SZLJITO#";

/// 0: Empty
/// 1: Any
/// 2-8: S, Z, L, J, I, T, O
/// 9: Garbage
pub struct CellTypeId(pub u8);

impl CellTypeId {
    pub fn to_piece(&self) -> Option<Piece> {
        if self.0 < 2 || 8 < self.0 {
            return None;
        }
        Some(Piece::from((self.0 - 2) as usize))
    }
    pub fn is_valid(&self) -> bool { self.0 <= 9 }
    pub fn to_char(&self) -> char {
        assert!(self.is_valid());
        CELL_CHARS.chars().nth(self.0 as usize).unwrap()
    }
    pub fn from_char(c: char) -> Self { Cell::from(c).into() }
}

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

impl Piece {
    pub fn default_spec(self) -> &'static PieceSpec<'static> {
        &DEFAULT_PIECE_SPEC_COLLECTION.get(self)
    }
    pub fn char(self) -> char {
        Cell::Block(Block::Piece(self)).char()
    }
    pub fn from_char(c: char) -> Result<Self, &'static str> {
        match Cell::from(c) {
            Cell::Block(Block::Piece(p)) => Ok(p),
            _ => Err("not piece char"),
        }
    }
}

pub const NUM_PIECES: usize = 7;

pub const PIECES: [Piece; NUM_PIECES] = [Piece::S, Piece::Z, Piece::L, Piece::J, Piece::I, Piece::T, Piece::O];

impl From<usize> for Piece {
    fn from(n: usize) -> Self {
        assert!(n < 7);
        PIECES[n]
    }
}

impl Piece {
    pub fn to_usize(&self) -> usize {
        match self {
            Piece::S => 0,
            Piece::Z => 1,
            Piece::L => 2,
            Piece::J => 3,
            Piece::I => 4,
            Piece::T => 5,
            Piece::O => 6,
        }
    }
}

impl From<Piece> for CellTypeId {
    fn from(p: Piece) -> Self { Self(2 + (p as u8)) }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Block {
    Any,
    Piece(Piece),
    Garbage,
}

impl From<Block> for CellTypeId {
    fn from(b: Block) -> Self {
        match b {
            Block::Any => Self(1),
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
            ' ' | '_' => Cell::Empty,
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

impl CellTrait for Cell {
    fn empty() -> Self { Self::Empty }
    fn any_block() -> Self { Self::Block(Block::Any) }
    fn is_empty(&self) -> bool { *self == Self::Empty }
    fn char(&self) -> char {
        let id = CellTypeId::from(*self);
        CELL_CHARS.chars().nth(id.0 as usize).unwrap()
    }
}

//--------------------------------------------------------------------------------------------------
// Grid Aliases
//--------------------------------------------------------------------------------------------------

type BasicGrid = grid::BasicGrid<Cell>;

type BitGridInt = u64;
type PrimBitGridConstantsStore = grid::bitgrid::PrimBitGridConstantsStore<BitGridInt>;
type PrimBitGrid<'a> = grid::bitgrid::PrimBitGrid<'a, BitGridInt, Cell>;
type BasicBitGrid<'a> = grid::bitgrid::BasicBitGrid<'a, BitGridInt, Cell>;

pub static DEFAULT_PRIM_GRID_CONSTANTS_STORE: Lazy<PrimBitGridConstantsStore> = Lazy::new(|| {
    let def = global_defaults();
    // Use the width of a playfield as the stride.
    let mut store = PrimBitGridConstantsStore::new(def.playfield_size.0);
    // For I piece.
    store.prepare_for_prim_bit_grid(Vec2(5, 5));
    // For other pieces.
    store.prepare_for_prim_bit_grid(Vec2(3, 3));
    // For playfield.
    store.prepare_for_bit_grid(def.playfield_size);
    store
});

#[derive(Clone, Debug, Eq)]
pub struct HybridGrid<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> {
    /// The field of BasicGrid is optional.
    /// If enabled, the piece types of each cell on the playfield are managed.
    /// By disabling this, we will get better performance and smaller memory usage.
    pub basic_grid: Option<BasicGrid>,
    pub bit_grid: BitGrid,
    phantom: PhantomData<fn() -> &'a ()>,
}

impl<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> HybridGrid<'a, BitGrid> {
    pub fn new(basic_grid: Option<BasicGrid>, bit_grid: BitGrid) -> Self {
        Self { basic_grid, bit_grid, phantom: PhantomData }
    }
    pub fn with_store(store: &'a PrimBitGridConstantsStore, size: Vec2, with_basic_grid: bool) -> Option<Self> {
        BitGrid::with_store(store, size).map(|bit_grid| {
            Self::new(if with_basic_grid { Some(BasicGrid::new(size)) } else { None }, bit_grid)
        })
    }
    pub fn disable_basic_grid(&mut self) {
        self.basic_grid = None;
    }
    pub fn put_fast<'b>(&mut self, pos: Vec2, other: &HybridGrid<'b, PrimBitGrid<'b>>) {
        if let Some(grid) = self.basic_grid.as_mut() {
            if let Some(other_grid) = other.basic_grid.as_ref() {
                grid.put(pos, other_grid);
            } else {
                grid.put(pos, &other.bit_grid);
            }
        }
        self.bit_grid.put_prim_bit_grid(pos, &other.bit_grid);
    }
    pub fn can_put_fast<'b>(&self, pos: Vec2, other: &HybridGrid<'b, PrimBitGrid<'b>>) -> bool {
        self.bit_grid.can_put_prim_bit_grid(pos, &other.bit_grid)
    }
    pub fn num_droppable_rows_fast<'b>(&self, pos: Vec2, other: &HybridGrid<'b, PrimBitGrid<'b>>) -> Y {
        self.bit_grid.num_droppable_rows_of_prim_bit_grid(pos, &other.bit_grid)
    }
    pub fn reachable_pos_fast<'b>(&self, pos: Vec2, other: &HybridGrid<'b, PrimBitGrid<'b>>, direction: Vec2) -> Vec2 {
        self.bit_grid.reachable_pos_of_prim_bit_grid(pos, &other.bit_grid, direction)
    }
}

impl<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> Grid<Cell> for HybridGrid<'a, BitGrid> {
    fn width(&self) -> X { self.bit_grid.width() }
    fn height(&self) -> X { self.bit_grid.height() }
    fn cell(&self, pos: Vec2) -> Cell {
        if let Some(g) = self.basic_grid.as_ref() {
            g.cell(pos)
        } else {
            self.bit_grid.cell(pos)
        }
    }
    fn set_cell(&mut self, pos: Vec2, cell: Cell) {
        self.basic_grid.as_mut().map(|g| g.set_cell(pos, cell));
        self.bit_grid.set_cell(pos, cell);
    }
    fn is_empty(&self) -> bool { self.bit_grid.is_empty() }
    fn fill_row(&mut self, y: Y, cell: Cell) {
        self.basic_grid.as_mut().map(|g| g.fill_row(y, cell));
        self.bit_grid.fill_row(y, cell);
    }
    fn fill_all(&mut self, cell: Cell) {
        self.basic_grid.as_mut().map(|g| g.fill_all(cell));
        self.bit_grid.fill_all(cell);
    }
    fn fill_top(&mut self, n: Y, cell: Cell) {
        self.basic_grid.as_mut().map(|g| g.fill_top(n, cell));
        self.bit_grid.fill_top(n, cell);
    }
    fn fill_bottom(&mut self, n: Y, cell: Cell) {
        self.basic_grid.as_mut().map(|g| g.fill_bottom(n, cell));
        self.bit_grid.fill_bottom(n, cell);
    }
    fn is_row_filled(&self, y: Y) -> bool { self.bit_grid.is_row_filled(y) }
    fn is_row_empty(&self, y: Y) -> bool { self.bit_grid.is_row_empty(y) }
    fn is_col_filled(&self, x: X) -> bool { self.bit_grid.is_col_filled(x) }
    fn is_col_empty(&self, x: X) -> bool { self.bit_grid.is_col_empty(x) }
    fn swap_rows(&mut self, y1: Y, y2: Y) {
        self.basic_grid.as_mut().map(|g| g.swap_rows(y1, y2));
        self.bit_grid.swap_rows(y1, y2);
    }
    fn num_blocks_of_row(&self, y: Y) -> usize { self.bit_grid.num_blocks_of_row(y) }
    fn num_blocks(&self) -> usize { self.bit_grid.num_blocks() }
}

impl<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> fmt::Display for HybridGrid<'a, BitGrid> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { self.format(f) }
}

impl<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> PartialEq for HybridGrid<'a, BitGrid> {
    fn eq(&self, other: &Self) -> bool { self.basic_grid == other.basic_grid }
}

impl<'a, BitGrid: BitGridTrait<'a, BitGridInt, Cell>> Hash for HybridGrid<'a, BitGrid> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.basic_grid.hash(state); }
}

//--------------------------------------------------------------------------------------------------
// Orientation
//--------------------------------------------------------------------------------------------------

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
    pub fn is_even(self) -> bool { self.0 % 2 == 0 }
    pub fn is_odd(self) -> bool { self.0 % 2 == 1 }
}

//--------------------------------------------------------------------------------------------------
// Placement
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Placement {
    pub orientation: Orientation,
    pub pos: Vec2,
}

impl Placement {
    pub fn new(orientation: Orientation, pos: Vec2) -> Self {
        Self { orientation, pos }
    }
    /// Manhattan distance.
    /// ```txt
    /// factors = (fo, fx, fy) (default: (1, 1, 1))
    /// distance = do * fo + dx * fx + dy * fy
    /// ```
    pub fn distance(&self, other: &Placement, factors: Option<(usize, usize, usize)>) -> usize {
        let dp = self.pos - other.pos;
        let (fo, fx, fy) = factors.unwrap_or((1, 1, 1));
        (dp.0.abs() as usize) * fx
            + (dp.1.abs() as usize) * fy
            + ((self.orientation.id() as i8 - other.orientation.id() as i8).abs() as usize) * fo
    }
    pub fn normalize(&self, piece: Piece) -> Placement {
        let alts = helper::get_alternative_placements(piece, self);
        match alts.iter().min_by(|p1, p2| p1.orientation.id().cmp(&p2.orientation.id())) {
            Some(p) => if self.orientation.id() < p.orientation.id() {
                self.clone()
            } else {
                *p
            }
            None => self.clone(),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Move
//--------------------------------------------------------------------------------------------------

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
pub struct MovePathItem {
    pub by: Move,
    pub placement: Placement,
}

impl MovePathItem {
    pub fn new(by: Move, placement: Placement) -> Self {
        Self { by, placement }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MoveTransition {
    pub placement: Placement,
    pub hint: Option<MovePathItem>,
}

impl MoveTransition {
    pub fn new(placement: Placement, hint: Option<MovePathItem>) -> Self {
        Self { placement, hint }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct MovePath {
    pub initial_placement: Placement,
    pub items: Vec<MovePathItem>,
}

impl MovePath {
    pub fn new(initial_placement: Placement) -> Self {
        Self {
            initial_placement,
            items: Vec::new(),
        }
    }
    pub fn len(&self) -> usize { self.items.len() }
    pub fn push(&mut self, item: MovePathItem) { self.items.push(item); }
    pub fn pop(&mut self) -> Option<MovePathItem> { self.items.pop() }
    pub fn get(&self, i: usize) -> Option<&MovePathItem> { self.items.get(i) }
    pub fn get_mut(&mut self, i: usize) -> Option<&mut MovePathItem> { self.items.get_mut(i) }
    pub fn last(&self) -> Option<&MovePathItem> { self.items.last() }
    pub fn iter(&self) -> std::slice::Iter<MovePathItem> { self.items.iter() }
    pub fn append(&mut self, other: &MovePath) {
        if let Some(item) = self.items.last() {
            debug_assert_eq!(item.placement, other.initial_placement);
        }
        self.items.extend(&other.items);
    }
    pub fn merge_or_push(&mut self, item: MovePathItem) {
        if let Some(last) = self.items.last_mut() {
            if let Some(mv) = last.by.merge(item.by) {
                last.by = mv;
                last.placement = item.placement;
                return;
            }
        }
        self.items.push(item);
    }
    pub fn normalize(&self) -> Self {
        let mut r = Self::new(self.initial_placement);
        for item in self.iter() {
            r.merge_or_push(*item);
        }
        r
    }
    pub fn last_transition(&self, use_hint: bool) -> Option<MoveTransition> {
        let len = self.len();
        if len == 0 {
            return None;
        }
        Some(MoveTransition::new(
            self.items[len - 1].placement,
            if use_hint {
                Some(MovePathItem::new(
                    self.items[len - 1].by,
                    if len == 1 {
                        self.initial_placement
                    } else {
                        self.items[len - 2].placement
                    },
                ))
            } else {
                None
            },
        ))
    }
}

//--------------------------------------------------------------------------------------------------
// Line Clear
//--------------------------------------------------------------------------------------------------

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
    pub fn tetris() -> Self { Self::new(4, None) }
    pub fn tst() -> Self { Self::new(3, Some(TSpin::Standard)) }
    pub fn tsd() -> Self { Self::new(2, Some(TSpin::Standard)) }
    pub fn tss() -> Self { Self::new(1, Some(TSpin::Standard)) }
    pub fn tsmd() -> Self { Self::new(2, Some(TSpin::Mini)) }
    pub fn tsms() -> Self { Self::new(1, Some(TSpin::Mini)) }
    pub fn tsmz() -> Self { Self::new(0, Some(TSpin::Mini)) }
    pub fn is_normal(&self) -> bool { self.tspin.is_none() }
    pub fn is_tspin(&self) -> bool { self.tspin.map_or(false, |t| t == TSpin::Standard) }
    pub fn is_tspin_mini(&self) -> bool { self.tspin.map_or(false, |t| t == TSpin::Mini) }
    pub fn is_tetris(&self) -> bool { self.is_normal() && self.num_lines == 4 }
    pub fn is_tst(&self) -> bool { self.is_tspin() && self.num_lines == 3 }
    pub fn is_tsd(&self) -> bool { self.is_tspin() && self.num_lines == 2 }
    pub fn is_tss(&self) -> bool { self.is_tspin() && self.num_lines == 1 }
    pub fn is_tsmd(&self) -> bool { self.is_tspin_mini() && self.num_lines == 2 }
    pub fn is_tsms(&self) -> bool { self.is_tspin_mini() && self.num_lines == 1 }
    pub fn is_tsmz(&self) -> bool { self.is_tspin_mini() && self.num_lines == 0 }
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

//--------------------------------------------------------------------------------------------------
// Game Rule
//--------------------------------------------------------------------------------------------------

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
        const LOCK_OUT         = 0b0010;
        const PARTIAL_LOCK_OUT = 0b0100;
        const GARBAGE_OUT      = 0b1000;
    }
}

impl Default for LossConditions {
    fn default() -> Self {
        Self::BLOCK_OUT | Self::LOCK_OUT | Self::GARBAGE_OUT
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct GameRules {
    pub rotation_mode: RotationMode,
    pub tspin_judgement_mode: TSpinJudgementMode,
    pub loss_conds: LossConditions,
}

//--------------------------------------------------------------------------------------------------
// PieceSpec
//--------------------------------------------------------------------------------------------------

fn srs_offset_data_i() -> Vec<Vec<(X, Y)>> {
    vec![
        vec![(0, 0), (-1, 0), (2, 0), (-1, 0), (2, 0)],
        vec![(-1, 0), (0, 0), (0, 0), (0, 1), (0, -2)],
        vec![(-1, 1), (1, 1), (-2, 1), (1, 0), (-2, 0)],
        vec![(0, 1), (0, 1), (0, 1), (0, -1), (0, 2)],
    ]
}

fn srs_offset_data_o() -> Vec<Vec<(X, Y)>> {
    vec![
        vec![(0, 0)],
        vec![(0, -1)],
        vec![(-1, -1)],
        vec![(-1, 0)],
    ]
}

fn srs_offset_data_others() -> Vec<Vec<(X, Y)>> {
    vec![
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (1, 0), (1, -1), (0, 2), (1, 2)],
        vec![(0, 0), (0, 0), (0, 0), (0, 0), (0, 0)],
        vec![(0, 0), (-1, 0), (-1, -1), (0, 2), (-1, 2)],
    ]
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PieceSpec<'a> {
    pub piece: Piece,
    /// The index of Vec is orientation.
    pub grids: Vec<HybridGrid<'a, PrimBitGrid<'a>>>,
    pub initial_placement: Placement,
    /// The index of outer Vec is orientation.
    pub srs_offset_data: Vec<Vec<(X, Y)>>,
}

impl<'a> PieceSpec<'a> {
    fn new(store: &'a PrimBitGridConstantsStore, piece: Piece, size: (X, Y), block_pos_list: Vec<(X, Y)>,
           initial_pos: (X, Y), srs_offset_data: Vec<Vec<(X, Y)>>) -> Self {
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
        let mut grids = Vec::with_capacity(basic_grids.len());
        for basic_grid in basic_grids {
            let mut g = PrimBitGrid::with_store(store, size.into()).unwrap();
            g.put((0, 0).into(), &basic_grid);
            grids.push(HybridGrid::new(Some(basic_grid), g));
        }
        Self {
            piece,
            grids,
            initial_placement: Placement::new(ORIENTATION_0, initial_pos.into()),
            srs_offset_data,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PieceSpecCollection<'a> {
    specs: Vec<PieceSpec<'a>>,
}

impl<'a> PieceSpecCollection<'a> {
    pub fn new(specs: Vec<PieceSpec<'a>>) -> Self {
        Self { specs }
    }
    pub fn get(&self, p: Piece) -> &PieceSpec<'a> {
        self.specs.get(p as usize).unwrap()
    }
}

struct PieceSpecBuilder<'a> {
    store: &'a PrimBitGridConstantsStore,
}

impl<'a> PieceSpecBuilder<'a> {
    pub fn new(store: &'a PrimBitGridConstantsStore) -> Self {
        Self { store }
    }
    pub fn piece_specs(&self) -> PieceSpecCollection<'a> {
        PieceSpecCollection::new(vec![
            self.piece_s(),
            self.piece_z(),
            self.piece_l(),
            self.piece_j(),
            self.piece_i(),
            self.piece_t(),
            self.piece_o(),
        ])
    }
    /// ```ignore
    /// +---+
    /// | SS|
    /// |SS |
    /// |   |
    /// +---+
    /// ```
    fn piece_s(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::S,
            (3, 3),
            vec![(0, 1), (1, 1), (1, 2), (2, 2)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    /// ```ignore
    /// +---+
    /// |ZZ |
    /// | ZZ|
    /// |   |
    /// +---+
    /// ```
    fn piece_z(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::Z,
            (3, 3),
            vec![(0, 2), (1, 1), (1, 2), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    /// ```ignore
    /// +---+
    /// |  L|
    /// |LLL|
    /// |   |
    /// +---+
    /// ```
    fn piece_l(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::L,
            (3, 3),
            vec![(0, 1), (1, 1), (2, 1), (2, 2)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    /// ```ignore
    /// +---+
    /// |J  |
    /// |JJJ|
    /// |   |
    /// +---+
    /// ```
    fn piece_j(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::J,
            (3, 3),
            vec![(0, 1), (0, 2), (1, 1), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    /// ```ignore
    /// +-----+
    /// |     |
    /// |     |
    /// | IIII|
    /// |     |
    /// |     |
    /// +-----+
    /// ```
    fn piece_i(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::I,
            (5, 5),
            vec![(1, 2), (2, 2), (3, 2), (4, 2)],
            (2, 17),
            srs_offset_data_i(),
        )
    }
    /// ```ignore
    /// +---+
    /// | T |
    /// |TTT|
    /// |   |
    /// +---+
    /// ```
    fn piece_t(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::T,
            (3, 3),
            vec![(0, 1), (1, 1), (1, 2), (2, 1)],
            (3, 18),
            srs_offset_data_others(),
        )
    }
    /// ```ignore
    /// +---+
    /// | OO|
    /// | OO|
    /// |   |
    /// +---+
    /// ```
    fn piece_o(&self) -> PieceSpec<'a> {
        PieceSpec::new(
            self.store,
            Piece::O,
            (3, 3),
            vec![(1, 1), (1, 2), (2, 1), (2, 2)],
            (3, 18),
            srs_offset_data_o(),
        )
    }
}

pub static DEFAULT_PIECE_SPEC_COLLECTION: Lazy<PieceSpecCollection> = Lazy::new(|| {
    let b = PieceSpecBuilder::new(&DEFAULT_PRIM_GRID_CONSTANTS_STORE);
    b.piece_specs()
});

impl Default for &PieceSpecCollection<'static> {
    fn default() -> Self { &DEFAULT_PIECE_SPEC_COLLECTION }
}

//--------------------------------------------------------------------------------------------------
// FallingPiece
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FallingPiece<'a> {
    pub piece_spec: &'a PieceSpec<'a>,
    pub placement: Placement,
    pub move_path: MovePath,
}

impl<'a> FallingPiece<'a> {
    pub fn new(piece_spec: &'a PieceSpec, placement: Placement) -> Self {
        Self { piece_spec, placement, move_path: MovePath::new(placement) }
    }
    pub fn new_with_one_path_item(piece_spec: &'a PieceSpec, src: Placement, mv: Move, dst: Placement) -> Self {
        let mut fp = Self::new(piece_spec, dst);
        fp.move_path.initial_placement = src;
        fp.move_path.items.push(MovePathItem::new(mv, dst));
        fp
    }
    pub fn new_with_last_move_transition(piece_spec: &'a PieceSpec, mt: &MoveTransition) -> Self {
        if let Some(hint) = mt.hint {
            Self::new_with_one_path_item(piece_spec, hint.placement, hint.by, mt.placement)
        } else {
            Self::new(piece_spec, mt.placement)
        }
    }
    pub fn spawn(piece_spec: &'a PieceSpec, pf: Option<&Playfield>) -> Self {
        let mut fp = Self::new(piece_spec, piece_spec.initial_placement);
        if let Some(pf) = pf {
            if !pf.can_put(&fp) {
                fp.placement.pos.1 += 1;
                fp.move_path.initial_placement.pos.1 += 1;
            }
        }
        fp
    }
    pub fn piece(&self) -> Piece { self.piece_spec.piece }
    pub fn grid(&self) -> &'a HybridGrid<PrimBitGrid<'a>> {
        &self.piece_spec.grids[self.placement.orientation.id() as usize]
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
                if !pf.can_drop_n(self, n as Y) {
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
        self.move_path.push(MovePathItem::new(mv, self.placement));
        true
    }
    pub fn rollback(&mut self) -> bool {
        if let Some(_) = self.move_path.pop() {
            self.placement = self.move_path.last()
                .map_or(self.move_path.initial_placement, |item| { item.placement });
            true
        } else {
            false
        }
    }
    pub fn is_last_move_rotation(&self) -> bool {
        if let Some(item) = self.move_path.items.last() {
            if let Move::Rotate(_) = item.by {
                return true;
            }
        }
        false
    }
    pub fn last_move_transition(&self, use_hint: bool) -> Option<MoveTransition> {
        self.move_path.last_transition(use_hint)
    }
}

//--------------------------------------------------------------------------------------------------
// Playfield
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Playfield<'a> {
    pub grid: HybridGrid<'a, BasicBitGrid<'a>>,
    pub visible_height: Y,
}

impl<'a> Playfield<'a> {
    pub fn new(store: &'a PrimBitGridConstantsStore, size: Vec2, with_basic_grid: bool, visible_height: Y) -> Option<Self> {
        HybridGrid::with_store(store, size, with_basic_grid).map(|grid| Self { grid, visible_height })
    }
    pub fn width(&self) -> X { self.grid.width() }
    pub fn height(&self) -> X { self.grid.height() }
    pub fn is_empty(&self) -> bool { self.grid.is_empty() }
    pub fn set_rows_with_strs(&mut self, pos: Vec2, rows: &[&str]) {
        self.grid.set_rows_with_strs(pos, rows);
    }
    // If garbage out, `true` will be returned.
    pub fn append_garbage(&mut self, gap_x_list: &[X]) -> bool {
        let ok = self.grid.insert_rows(0, Cell::Block(Block::Garbage), gap_x_list.len() as Y);
        for (y, x) in gap_x_list.iter().enumerate() {
            self.grid.set_cell((*x, y as Y).into(), Cell::Empty);
        }
        !ok
    }
    pub fn can_put(&self, fp: &FallingPiece) -> bool {
        self.grid.can_put_fast(fp.placement.pos, fp.grid())
    }
    pub fn num_droppable_rows(&self, fp: &FallingPiece) -> Y {
        self.grid.num_droppable_rows_fast(fp.placement.pos, fp.grid())
    }
    pub fn num_shiftable_cols(&self, fp: &FallingPiece, to_right: bool) -> X {
        let p = self.grid.reachable_pos_fast(fp.placement.pos, fp.grid(),
                                             if to_right { (1, 0) } else { (-1, 0) }.into());
        let r = if to_right { p.0 - fp.placement.pos.0 } else { fp.placement.pos.0 - p.0 };
        debug_assert!(r >= 0);
        r as X
    }
    pub fn can_drop(&self, fp: &FallingPiece) -> bool {
        self.grid.can_put_fast(fp.placement.pos + (0, -1).into(), fp.grid())
    }
    pub fn can_drop_n(&self, fp: &FallingPiece, n: Y) -> bool {
        assert!(n > 0);
        n <= self.grid.num_droppable_rows_fast(fp.placement.pos, fp.grid())
    }
    pub fn can_raise_n(&self, fp: &FallingPiece, n: Y) -> bool {
        assert!(n > 0);
        for i in 0..n {
            let y = fp.placement.pos.1 + i + 1;
            if !self.grid.can_put_fast((fp.placement.pos.0, y).into(), fp.grid()) {
                return false;
            }
        }
        true
    }
    pub fn can_move_horizontally(&self, fp: &FallingPiece, n: X) -> bool {
        let to_right = n > 0;
        let end = if to_right { n } else { -n };
        for dx in 1..=end {
            let x = fp.placement.pos.0 + if to_right { dx } else { -dx };
            if !self.grid.can_put_fast((x, fp.placement.pos.1).into(), fp.grid()) {
                return false;
            }
        }
        true
    }
    pub fn check_rotation(&self, mode: RotationMode, fp: &FallingPiece, cw: bool) -> Option<Placement> {
        match mode {
            RotationMode::Srs => self.check_rotation_by_srs(fp, cw),
        }
    }
    pub fn check_rotation_by_srs(&self, fp: &FallingPiece, cw: bool) -> Option<Placement> {
        let next_orientation: Orientation = fp.placement.orientation.rotate(if cw { 1 } else { -1 });
        let spec = fp.piece_spec;
        let next_grid = &spec.grids[next_orientation.id() as usize];
        let offsets1 = &spec.srs_offset_data[fp.placement.orientation.id() as usize];
        let offsets2 = &spec.srs_offset_data[next_orientation.id() as usize];
        for i in 0..offsets1.len() {
            let p = fp.placement.pos + offsets1[i].into() - offsets2[i].into();
            if self.grid.can_put_fast(p, next_grid) {
                return Some(Placement::new(next_orientation, p));
            }
        }
        None
    }
    pub fn check_reverse_rotation(&self, mode: RotationMode, fp: &FallingPiece, cw: bool) -> Vec<Placement> {
        match mode {
            RotationMode::Srs => self.check_reverse_rotation_by_srs(fp, cw),
        }
    }
    pub fn check_reverse_rotation_by_srs(&self, fp: &FallingPiece, cw: bool) -> Vec<Placement> {
        let prev_orientation: Orientation = fp.placement.orientation.rotate(if cw { -1 } else { 1 });
        let spec = fp.piece_spec;
        let prev_grid = &spec.grids[prev_orientation.id() as usize];
        let offsets1 = &spec.srs_offset_data[prev_orientation.id() as usize];
        let offsets2 = &spec.srs_offset_data[fp.placement.orientation.id() as usize];
        let mut r = Vec::new();
        for i in 0..offsets1.len() {
            let p = fp.placement.pos - offsets1[i].into() + offsets2[i].into();
            if self.grid.can_put_fast(p, prev_grid) {
                let prev_fp = FallingPiece::new(fp.piece_spec, Placement::new(prev_orientation, p));
                if let Some(pp) = self.check_rotation_by_srs(&prev_fp, cw) {
                    if pp == fp.placement {
                        r.push(Placement::new(prev_orientation, p));
                    }
                }
            }
        }
        r
    }
    // This method doesn't consider whether the game is over or not.
    pub fn can_lock(&self, fp: &FallingPiece) -> bool { self.can_put(fp) && !self.can_drop(fp) }
    pub fn check_tspin(&self, fp: &FallingPiece, mode: TSpinJudgementMode) -> Option<TSpin> {
        debug_assert!(self.can_lock(fp));
        debug_assert_eq!(TSpinJudgementMode::PuyoPuyoTetris, mode);
        if fp.piece() != Piece::T || !fp.is_last_move_rotation() {
            return None;
        }
        let mut num_corners = 0;
        let mut num_pointing_side_corners = 0;
        for dy in &[0, 2] {
            for dx in &[0, 2] {
                let dx = *dx;
                let dy = *dy;
                let pos: Vec2 = (fp.placement.pos.0 + dx, fp.placement.pos.1 + dy).into();
                let is_wall = pos.0 < 0 || pos.1 < 0 || pos.0 >= self.width() as X || pos.1 >= self.height() as Y;
                if is_wall || !self.grid.cell(pos.into()).is_empty() {
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
                } else if let Some(mt) = fp.last_move_transition(true) {
                    if let Some(hint) = mt.hint {
                        let src = &hint.placement;
                        let dst = &mt.placement;
                        let is_shifted = src.pos.0 != dst.pos.0;
                        let num_rows = src.pos.1 - dst.pos.1;
                        if num_rows == 2 {
                            if is_shifted {
                                Some(TSpin::Standard)
                            } else {
                                Some(TSpin::Mini) // Neo
                            }
                        } else {
                            Some(TSpin::Mini)
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
        let mut tmp_grid = self.grid.bit_grid.clone();
        tmp_grid.put_prim_bit_grid(fp.placement.pos, &fp.grid().bit_grid);
        LineClear::new(tmp_grid.num_filled_rows() as u8, self.check_tspin(fp, mode))
    }
    pub fn check_lock_out(&self, fp: &FallingPiece) -> Option<LockOutType> {
        let bottom = fp.placement.pos.1 + fp.grid().bottom_padding() as Y;
        if bottom >= self.visible_height as Y {
            return Some(LockOutType::LockOut);
        }
        let top = fp.placement.pos.1 + fp.grid().height() as Y - fp.grid().top_padding() as Y - 1;
        if top >= self.visible_height as Y {
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
        Some(LineClear::new(num_cleared_line as u8, tspin))
    }
    /// The return placements can include unreachable placements.
    /// These also includes all alternative placements.
    pub fn search_lockable_placements(&self, spec: &PieceSpec) -> Vec<Placement> {
        let max_padding = match spec.piece {
            Piece::I => 2,
            _ => 1,
        };
        let yend = (self.grid.height() - self.grid.top_padding()) as Y;
        let piece_grids = [
            &spec.grids[ORIENTATION_0.id() as usize],
            &spec.grids[ORIENTATION_1.id() as usize],
            &spec.grids[ORIENTATION_2.id() as usize],
            &spec.grids[ORIENTATION_3.id() as usize],
        ];
        let mut r: Vec<Placement> = Vec::new();
        for y in -max_padding..=yend {
            for x in -max_padding..=(self.grid.width() as X - max_padding) {
                for o in &ORIENTATIONS {
                    let g = piece_grids[o.id() as usize];
                    let can_put = self.grid.can_put_fast((x, y).into(), g);
                    if !can_put {
                        continue;
                    }
                    let can_drop = self.grid.can_put_fast((x, y - 1).into(), g);
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

impl Default for Playfield<'static> {
    fn default() -> Self {
        let def = global_defaults();
        Self::new(&DEFAULT_PRIM_GRID_CONSTANTS_STORE, def.playfield_size, true, def.playfield_visible_height).unwrap()
    }
}

//--------------------------------------------------------------------------------------------------
// NextPieces
//--------------------------------------------------------------------------------------------------

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
    pub fn remove_invisible(&mut self) {
        self.pieces = self.pieces.iter().take(self.visible_num).copied().collect();
    }
}

impl Default for NextPieces {
    fn default() -> Self { Self::new(global_defaults().num_visible_next_pieces) }
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

#[derive(Clone, Debug)]
pub struct RandomPieceGenerator<R: rand::Rng + ?Sized> {
    rng: R,
}

impl<R: rand::Rng + Sized> RandomPieceGenerator<R> {
    pub fn new(rng: R) -> Self { Self { rng } }
    pub fn generate(&mut self) -> Vec<Piece> {
        let mut ps = PIECES.clone();
        ps.shuffle(&mut self.rng);
        ps.to_vec()
    }
}

//--------------------------------------------------------------------------------------------------
// Statistics
//--------------------------------------------------------------------------------------------------

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
            let dc = *count - other.get(lc);
            if dc > 0 {
                r.add(lc, dc);
            }
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
            let dc = *count - other.get(*cont_count);
            if dc > 0 {
                r.add(*cont_count, dc);
            }
        }
        r
    }
}

#[derive(Copy, Clone, Debug)]
pub enum StatisticsEntryType {
    LineClear(LineClear),
    Combo(Count),
    MaxCombos,
    Btb(Count),
    MaxBtbs,
    PerfectClear,
    Hold,
    Lock,
}

impl fmt::Display for StatisticsEntryType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StatisticsEntryType::LineClear(lc) => write!(f, "{}", lc),
            StatisticsEntryType::Combo(n) => write!(f, "combo[{}]", n),
            StatisticsEntryType::MaxCombos => write!(f, "max combos"),
            StatisticsEntryType::Btb(n) => write!(f, "btb[{}]", n),
            StatisticsEntryType::MaxBtbs => write!(f, "max btbs"),
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
    pub fn get(&self, t: StatisticsEntryType) -> Count {
        match t {
            StatisticsEntryType::LineClear(lc) => self.line_clear.get(&lc),
            StatisticsEntryType::Combo(n) => self.combo.get(n),
            StatisticsEntryType::MaxCombos => self.combo.max(),
            StatisticsEntryType::Btb(n) => self.btb.get(n),
            StatisticsEntryType::MaxBtbs => self.btb.max(),
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

//--------------------------------------------------------------------------------------------------
// GameState
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct GameState<'a> {
    pub playfield: Playfield<'a>,
    pub next_pieces: NextPieces,
    pub falling_piece: Option<FallingPiece<'a>>,
    pub hold_piece: Option<Piece>,
    pub can_hold: bool,
    pub num_combos: Option<Count>,
    pub num_btbs: Option<Count>,
    pub game_over_reason: LossConditions,
}

impl<'a> GameState<'a> {
    pub fn is_game_over(&self) -> bool { !self.game_over_reason.is_empty() }
}

impl Default for GameState<'static> {
    fn default() -> Self {
        Self {
            playfield: Default::default(),
            next_pieces: Default::default(),
            falling_piece: None,
            hold_piece: None,
            can_hold: true,
            num_combos: None,
            num_btbs: None,
            game_over_reason: LossConditions::empty(),
        }
    }
}

//--------------------------------------------------------------------------------------------------
// Game
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Game<'a> {
    pub piece_specs: &'a PieceSpecCollection<'a>,
    pub rules: GameRules,
    pub state: GameState<'a>,
    pub stats: Statistics,
}

impl<'a> Game<'a> {
    pub fn new(piece_specs: &'a PieceSpecCollection<'a>, rules: GameRules, state: GameState<'a>, stats: Statistics) -> Self {
        Self {
            piece_specs,
            rules,
            state,
            stats,
        }
    }
    /// The performance becomes better but piece information in the playfield will be lacked.
    pub fn fast_mode(&mut self) {
        self.state.playfield.grid.disable_basic_grid();
    }
    pub fn get_cell(&self, pos: Vec2) -> Cell {
        let s = &self.state;
        let mut cell = if let Some(fp) = s.falling_piece.as_ref() {
            let grid = fp.grid();
            let grid_pos = pos - fp.placement.pos;
            if grid.is_inside(grid_pos) {
                grid.cell(grid_pos.into())
            } else {
                Cell::Empty
            }
        } else {
            Cell::Empty
        };
        if cell == Cell::Empty {
            cell = s.playfield.grid.cell(pos.into());
        }
        cell
    }
    pub fn should_supply_next_pieces(&self) -> bool {
        self.state.next_pieces.should_supply()
    }
    pub fn supply_next_pieces(&mut self, pieces: &[Piece]) {
        self.state.next_pieces.supply(pieces);
    }
    /// This method should be called right after `new()`.
    /// `Err` will be returned when there are no next pieces.
    pub fn setup_falling_piece(&mut self, next: Option<Piece>) -> Result<(), &'static str> {
        let s = &mut self.state;

        if s.falling_piece.is_some() {
            return Err("falling piece already exists");
        }

        let p = if let Some(next) = next {
            next
        } else {
            if s.next_pieces.is_empty() {
                return Err("no next pieces");
            }
            s.next_pieces.pop().unwrap()
        };

        let fp = FallingPiece::spawn(self.piece_specs.get(p), Some(&s.playfield));
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
        if fp.apply_move(mv, &self.state.playfield, self.rules.rotation_mode) {
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
                    if self.rules.loss_conds.contains(LossConditions::LOCK_OUT) {
                        s.game_over_reason |= LossConditions::LOCK_OUT | LossConditions::PARTIAL_LOCK_OUT;
                    }
                }
                LockOutType::PartialLockOut => {
                    if self.rules.loss_conds.contains(LossConditions::PARTIAL_LOCK_OUT) {
                        s.game_over_reason |= LossConditions::PARTIAL_LOCK_OUT;
                    }
                }
            }
        }
        let line_clear = pf.lock(fp, self.rules.tspin_judgement_mode);
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
        let p = s.falling_piece.as_ref().unwrap().piece_spec.piece;
        s.falling_piece = None;
        let r = self.setup_falling_piece(self.state.hold_piece);
        self.state.hold_piece = Some(p);
        self.state.can_hold = false;
        self.stats.hold += 1;
        Ok(r.is_ok())
    }
    pub fn search_moves(&self, searcher: &mut impl move_search::MoveSearcher) -> Result<move_search::SearchResult, &'static str> {
        let s = &self.state;
        if s.falling_piece.is_none() {
            return Err("no falling piece");
        }
        let fp = s.falling_piece.as_ref().unwrap();
        let pf = &s.playfield;
        let conf = move_search::SearchConfiguration::new(pf, fp.piece_spec, fp.placement, self.rules.rotation_mode);
        Ok(searcher.search(&conf))
    }
    #[deprecated(note = "Use helper::MoveDecisionHelper.")]
    pub fn get_move_candidates(&self) -> Result<HashSet<MoveTransition>, &'static str> {
        let s = &self.state;
        if s.falling_piece.is_none() {
            return Err("no falling piece");
        }
        let r = helper::get_move_candidates(&s.playfield, s.falling_piece.as_ref().unwrap(), &self.rules);
        Ok(r)
    }
    pub fn get_move_decision_helper(&self, stuff: Option<Rc<MoveDecisionStuff>>) -> Result<MoveDecisionHelper, &'static str> {
        MoveDecisionHelper::with_game(self, stuff)
    }
    pub fn get_almost_good_move_path(&self, last_transition: &MoveTransition) -> Result<MovePath, &'static str> {
        let fp = if let Some(fp) = self.state.falling_piece.as_ref() {
            fp
        } else {
            return Err("no falling piece");
        };
        let dst = if let Some(hint) = last_transition.hint.as_ref() { &hint.placement } else { &last_transition.placement };
        if let Some(mut path) = helper::get_almost_good_move_path(self.rules.rotation_mode, &self.state.playfield, fp, dst) {
            if let Some(hint) = last_transition.hint {
                path.merge_or_push(hint);
            }
            Ok(path)
        } else {
            Err("move path not found")
        }
    }
}

impl<'a> fmt::Display for Game<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = &self.state;
        let w = self.state.playfield.width() as usize;
        let h = self.state.playfield.visible_height as usize;
        let num_next = std::cmp::min(self.state.next_pieces.visible_num, self.state.next_pieces.len());
        write!(f, "[{}]", s.hold_piece.map_or(
            Cell::Empty, |p| { Cell::Block(Block::Piece(p)) }).char(),
        )?;
        write!(f, "{}", " ".repeat(w - num_next - 2))?;
        write!(f, "({})", s.falling_piece.as_ref().map_or(
            Cell::Empty, |fp| { Cell::Block(Block::Piece(fp.piece())) }).char(),
        )?;
        writeln!(f, "{}", s.next_pieces)?;
        writeln!(f, "--+{}+", "-".repeat(w))?;
        for i in 0..h {
            let y = h - 1 - i;
            write!(f, "{:02}|", y)?;
            for x in 0..w {
                let cell = self.get_cell((x as X, y as Y).into());
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
                    write!(f, "  {:6}  {}/{}", "COMBO", s.num_combos.unwrap_or(0), self.stats.get(StatisticsEntryType::MaxCombos))?;
                }
                10 => {
                    write!(f, "  {:6}  {}/{}", "BTB", s.num_combos.unwrap_or(0), self.stats.get(StatisticsEntryType::MaxBtbs))?;
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

//--------------------------------------------------------------------------------------------------
// MovePlayer
//--------------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct MovePlayer {
    path: MovePath,
    i: usize,
}

impl MovePlayer {
    pub fn new(path: MovePath) -> Self {
        Self { path, i: 0 }
    }
    pub fn is_end(&self) -> bool { self.i >= self.path.len() }
    pub fn step(&mut self, game: &mut Game) -> Result<bool, &'static str> {
        if self.is_end() {
            return Ok(false);
        }
        if game.state.falling_piece.is_none() {
            return Err("no falling piece");
        }
        let fp = game.state.falling_piece.as_ref().unwrap();
        let placement = if self.i == 0 {
            self.path.initial_placement
        } else {
            self.path.items[self.i - 1].placement
        };
        if fp.placement != placement {
            return Err("invalid placement");
        }
        let item = self.path.items[self.i];
        game.do_move(item.by)?;
        self.i += 1;
        Ok(true)
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
    fn test_grid_num_covered_empty_cells() {
        let mut grid = BasicGrid::new((10, 10).into());
        grid.set_cell((0, 5).into(), Cell::Block(Block::Any));
        grid.set_cell((0, 2).into(), Cell::Block(Block::Any));
        grid.set_cell((2, 1).into(), Cell::Block(Block::Any));
        assert_eq!(5, grid.num_covered_empty_cells());
    }

    #[test]
    fn test_falling_piece() {
        let pf = Playfield::default();
        let mut fp = FallingPiece::spawn(Piece::O.default_spec(), Some(&pf));
        assert!(fp.apply_move(Move::Shift(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Shift(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Rotate(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Rotate(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Drop(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Drop(1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Rotate(-1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Rotate(-1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Shift(-1), &pf, RotationMode::Srs));
        assert!(fp.apply_move(Move::Shift(-1), &pf, RotationMode::Srs));
        let path = fp.move_path.normalize();
        assert_eq!(Placement::new(ORIENTATION_0, (3, 18).into()), path.initial_placement);
        assert_eq!(vec![
            MovePathItem::new(Move::Shift(2), Placement::new(ORIENTATION_0, (5, 18).into())),
            MovePathItem::new(Move::Rotate(1), Placement::new(ORIENTATION_1, (5, 19).into())),
            MovePathItem::new(Move::Rotate(1), Placement::new(ORIENTATION_2, (6, 19).into())),
            MovePathItem::new(Move::Drop(2), Placement::new(ORIENTATION_2, (6, 17).into())),
            MovePathItem::new(Move::Rotate(-1), Placement::new(ORIENTATION_1, (5, 17).into())),
            MovePathItem::new(Move::Rotate(-1), Placement::new(ORIENTATION_0, (5, 16).into())),
            MovePathItem::new(Move::Shift(-2), Placement::new(ORIENTATION_0, (3, 16).into())),
        ], path.items);
    }

    #[test]
    fn test_random_piece_generator() {
        let mut rpg = RandomPieceGenerator::new(rand::thread_rng());
        let piece_set: std::collections::HashSet<Piece> = rpg.generate().iter().copied().collect::<_>();
        assert_eq!(NUM_PIECES, piece_set.len());
    }

    #[test]
    fn test_spawn_and_lock_out() {
        let mut pf = Playfield::default();
        pf.append_garbage(&[0].repeat(18));
        let mut fp = FallingPiece::spawn(Piece::O.default_spec(), Some(&pf));
        assert_eq!(18, fp.placement.pos.1);
        assert!(!pf.can_lock(&fp));
        assert!(fp.apply_move(Move::Drop(1), &pf, RotationMode::Srs));
        assert!(pf.can_lock(&fp));
        assert_eq!(None, pf.check_lock_out(&fp));
        pf.append_garbage(&[0]);
        let fp = FallingPiece::spawn(Piece::O.default_spec(), Some(&pf));
        assert_eq!(18, fp.placement.pos.1);
        assert!(pf.can_lock(&fp));
        assert_eq!(Some(LockOutType::PartialLockOut), pf.check_lock_out(&fp));
        pf.append_garbage(&[0]);
        let fp = FallingPiece::spawn(Piece::O.default_spec(), Some(&pf));
        assert_eq!(19, fp.placement.pos.1);
        assert!(pf.can_lock(&fp));
        assert_eq!(Some(LockOutType::LockOut), pf.check_lock_out(&fp));
        pf.append_garbage(&[0]);
        let fp = FallingPiece::spawn(Piece::O.default_spec(), Some(&pf));
        assert_eq!(19, fp.placement.pos.1);
        assert!(!pf.can_lock(&fp));
    }

    #[test]
    fn test_reverse_rotation_by_srs() {
        let mut pf = Playfield::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            "  @@@@@@@@",
            "   @@@@@@@",
            "@ @@@@@@@@",
        ]);
        let fp = FallingPiece::new(Piece::T.default_spec(), Placement::new(ORIENTATION_2, (0, 0).into()));
        let r_cw = pf.check_reverse_rotation_by_srs(&fp, true);
        assert_eq!(vec![
            Placement::new(ORIENTATION_1, (0, 0).into()),
            Placement::new(ORIENTATION_1, (-1, 1).into()),
        ], r_cw);
        let r_ccw = pf.check_reverse_rotation_by_srs(&fp, false);
        assert_eq!(vec![
            Placement::new(ORIENTATION_3, (0, 0).into()),
        ], r_ccw);
    }

    #[test]
    fn test_tspin_mini() {
        let mut pf = Playfield::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            " @@@@@@@@@",
        ]);
        let mut fp = FallingPiece::new(Piece::T.default_spec(), Placement::new(ORIENTATION_0, (0, 0).into()));
        assert!(fp.apply_move(Move::Rotate(1), &pf, RotationMode::Srs));
        assert_eq!(Placement::new(ORIENTATION_1, (-1, 0).into()), fp.placement);
        let tspin = pf.check_tspin(&fp, TSpinJudgementMode::PuyoPuyoTetris);
        assert_eq!(Some(TSpin::Mini), tspin);
    }

    #[test]
    fn test_tspin_neo() {
        let mut pf = Playfield::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            "       @@@",
            "         @",
            "        @@",
            "@@@@@@  @@",
            "@@@@@@@ @@",
        ]);
        let mut fp = FallingPiece::new(Piece::T.default_spec(), Placement::new(ORIENTATION_2, (6, 2).into()));
        assert!(fp.apply_move(Move::Rotate(1), &pf, RotationMode::Srs));
        assert_eq!(Placement::new(ORIENTATION_3, (6, 0).into()), fp.placement);
        let tspin = pf.check_tspin(&fp, TSpinJudgementMode::PuyoPuyoTetris);
        assert_eq!(Some(TSpin::Mini), tspin);
    }

    #[test]
    fn test_tspin_fin() {
        let mut pf = Playfield::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            "       @@@",
            "         @",
            "         @",
            "@@@@@@@  @",
            "@@@@@@@@ @",
        ]);
        let mut fp = FallingPiece::new(Piece::T.default_spec(), Placement::new(ORIENTATION_2, (6, 2).into()));
        assert!(fp.apply_move(Move::Rotate(1), &pf, RotationMode::Srs));
        assert_eq!(Placement::new(ORIENTATION_3, (7, 0).into()), fp.placement);
        let tspin = pf.check_tspin(&fp, TSpinJudgementMode::PuyoPuyoTetris);
        assert_eq!(Some(TSpin::Standard), tspin);
    }

    #[test]
    fn test_lockable() {
        let mut pf = Playfield::default();
        pf.set_rows_with_strs((0, 0).into(), &[
            " @@@@@@@@ ",
            " @@@@@@@@ ",
            " @@@@@@@@ ",
            " @@@@@@@@ ",
        ]);
        let ps = pf.search_lockable_placements(Piece::I.default_spec());
        assert!(ps.contains(&Placement::new(ORIENTATION_1, (-2, 0).into())));
        assert!(ps.contains(&Placement::new(ORIENTATION_3, (-2, -1).into())));
    }

    #[test]
    fn test_search_moves() {
        let mut game: Game = Default::default();
        game.supply_next_pieces(&[Piece::T]);
        assert_ok!(game.setup_falling_piece(None));
        let pf = &mut game.state.playfield;
        pf.set_rows_with_strs((0, 0).into(), &[
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
        let fp = game.state.falling_piece.as_ref().unwrap();
        let lockable = pf.search_lockable_placements(fp.piece().default_spec());

        let dst = Placement::new(ORIENTATION_3, (1, 0).into());
        assert!(lockable.iter().any(|p| { *p == dst }));

        for i in 0..=1 {
            let ret = match i {
                0 => game.search_moves(&mut move_search::bruteforce::BruteForceMoveSearcher::default()),
                1 => game.search_moves(&mut move_search::astar::AStarMoveSearcher::new(dst, false)),
                _ => panic!(),
            };
            assert_ok!(&ret);
            let ret = ret.unwrap();
            let path = ret.get(&dst);
            assert!(path.is_some());
            let path = path.unwrap();
            assert!(path.len() > 0);
        }
    }

    #[test]
    fn test_search_moves_2() {
        let mut game: Game = Default::default();
        game.supply_next_pieces(&[Piece::I]);
        assert_ok!(game.setup_falling_piece(None));
        let pf = &mut game.state.playfield;
        pf.set_rows_with_strs((0, 19).into(), &["       @  "]);
        pf.set_rows_with_strs((0, 0).into(), &[" @@@@@@@@ "].repeat(19));
        let all = game.get_move_candidates();
        assert!(all.is_ok());
        let all = all.unwrap();
        for mt in all.iter() {
            let r = game.search_moves(&mut move_search::astar::AStarMoveSearcher::new(mt.placement, false));
            assert!(r.is_ok());
            let r = r.unwrap();
            let path = r.get(&mt.placement);
            assert!(path.is_some());
        }
    }

    #[test]
    fn test_game() {
        let pieces = [
            Piece::O, Piece::T, Piece::I, Piece::J, Piece::L, Piece::S, Piece::Z,
            Piece::O, Piece::T, Piece::I, Piece::J, Piece::L, Piece::S, Piece::Z,
        ];

        let mut game: Game<'static> = Game::default();
        game.supply_next_pieces(&pieces);
        assert_ok!(game.setup_falling_piece(None));
        // Test simple TSD opener.
        // O
        assert_eq!(Piece::O, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.shift(-1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // T
        assert_eq!(Piece::T, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.hold());
        // I
        assert_eq!(Piece::I, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // J
        assert_eq!(Piece::J, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.rotate(-1));
        assert_ok!(game.shift(1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // L
        assert_eq!(Piece::L, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.rotate(1));
        assert_ok!(game.shift(-1, true));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // S
        assert_eq!(Piece::S, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.shift(1, false));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // Z
        assert_eq!(Piece::Z, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.shift(-2, false));
        assert_ok!(game.rotate(1));
        assert_ok!(game.firm_drop());
        assert_ok!(game.lock());
        // O
        assert_eq!(Piece::O, game.state.falling_piece.as_ref().unwrap().piece_spec.piece);
        assert_ok!(game.hold());
        // T
        assert_ok!(game.shift(1, true));
        assert_ok!(game.rotate(-1));
        assert_ok!(game.firm_drop());
        assert_ok!(game.rotate(-1));
        assert_ok!(game.lock());

        assert_eq!(r#"[O]   (T)IJLSZ
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

    #[test]
    fn test_move_player() {
        let mut game = Game::default();
        game.supply_next_pieces(&[Piece::O]);
        assert_ok!(game.setup_falling_piece(None));

        let pf = &game.state.playfield;
        let fp = game.state.falling_piece.as_ref().unwrap();

        let placements = game.state.playfield.search_lockable_placements(fp.piece_spec);
        let search_result = move_search::bruteforce::search_moves(
            &move_search::SearchConfiguration::new(&pf, fp.piece_spec, fp.placement, game.rules.rotation_mode),
            false,
        );
        let dst = placements[0];
        let mr = search_result.get(&dst).unwrap();

        let mut player = MovePlayer::new(mr);
        while !player.is_end() {
            assert_ok!(player.step(&mut game));
        }
        assert_eq!(dst, game.state.falling_piece.as_ref().unwrap().placement);
    }
}
