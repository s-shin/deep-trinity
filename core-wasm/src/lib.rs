extern crate wasm_bindgen;
extern crate core;
extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    if cfg!(feature = "console_error_panic_hook") {
        console_error_panic_hook::set_once();
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum Piece {
    S,
    Z,
    L,
    J,
    I,
    T,
    O,
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum Cell {
    EMPTY,
    ANY,
    S,
    Z,
    L,
    J,
    I,
    T,
    O,
    GARBAGE,
}

static CELLS: [Cell; 10] = [Cell::EMPTY, Cell::ANY, Cell::S, Cell::Z, Cell::L, Cell::J, Cell::I, Cell::T, Cell::O, Cell::GARBAGE];

impl From<core::Cell> for Cell {
    fn from(c: core::Cell) -> Self { CELLS[core::CellTypeId::from(c).0 as usize] }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum StatisticsEntryType {
    SINGLE,
    DOUBLE,
    TRIPLE,
    TETRIS,
    TST,
    TSD,
    TSS,
    TSMD,
    TSMS,
    MAX_COMBOS,
    MAX_BTBS,
    PERFECT_CLEAR,
    HOLD,
    LOCK,
}

#[wasm_bindgen]
pub struct Game {
    game: core::Game<core::StaticPieceGenerator>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            game: core::Game::new(Default::default()),
        }
    }
    pub fn width(&self) -> core::SizeX { self.game.state.playfield.width() }
    pub fn height(&self) -> core::SizeY { self.game.state.playfield.height() }
    #[wasm_bindgen(js_name = visibleHeight)]
    pub fn visible_height(&self) -> core::SizeY { self.game.state.playfield.visible_height }
    #[wasm_bindgen(js_name = getCell)]
    pub fn get_cell(&self, x: u8, y: u8) -> Cell { self.game.get_cell((x, y).into()).into() }
    #[wasm_bindgen(js_name = getHoldPiece)]
    pub fn get_hold_piece(&self) -> Option<u8> {
        self.game.state.hold_piece.map(|p| { p as u8 })
    }
    #[wasm_bindgen(js_name = getNextPieces)]
    pub fn get_next_pieces(&self) -> Box<[u8]> {
        let np = &self.game.state.next_pieces;
        np.iter()
            .take(np.visible_num)
            .map(|p| { *p as u8 })
            .collect::<Vec<u8>>()
            .into_boxed_slice()
    }
    #[wasm_bindgen(js_name = getCurrentNumCombos)]
    pub fn get_current_num_combos(&self) -> Option<core::Count> { self.game.state.num_combos }
    #[wasm_bindgen(js_name = getCurrentNumBTBs)]
    pub fn get_current_num_btbs(&self) -> Option<core::Count> { self.game.state.num_btbs }
    #[wasm_bindgen(js_name = getStatsCount)]
    pub fn get_stats_count(&self, t: StatisticsEntryType) -> core::Count {
        self.game.stats.get(match t {
            StatisticsEntryType::SINGLE => core::StatisticsEntryType::LineClear(core::LineClear::new(1, None)),
            StatisticsEntryType::DOUBLE => core::StatisticsEntryType::LineClear(core::LineClear::new(2, None)),
            StatisticsEntryType::TRIPLE => core::StatisticsEntryType::LineClear(core::LineClear::new(3, None)),
            StatisticsEntryType::TETRIS => core::StatisticsEntryType::LineClear(core::LineClear::new(4, None)),
            StatisticsEntryType::TST => core::StatisticsEntryType::LineClear(core::LineClear::new(3, Some(core::TSpin::Standard))),
            StatisticsEntryType::TSD => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Standard))),
            StatisticsEntryType::TSS => core::StatisticsEntryType::LineClear(core::LineClear::new(1, Some(core::TSpin::Standard))),
            StatisticsEntryType::TSMD => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Mini))),
            StatisticsEntryType::TSMS => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Mini))),
            StatisticsEntryType::MAX_COMBOS => core::StatisticsEntryType::MaxCombos,
            StatisticsEntryType::MAX_BTBS => core::StatisticsEntryType::MaxBtbs,
            StatisticsEntryType::PERFECT_CLEAR => core::StatisticsEntryType::PerfectClear,
            StatisticsEntryType::HOLD => core::StatisticsEntryType::Hold,
            StatisticsEntryType::LOCK => core::StatisticsEntryType::Lock,
        })
    }
    #[wasm_bindgen(js_name = appendNextPieces)]
    pub fn append_next_pieces(&mut self, pieces: &[u8]) {
        let mut ps: Vec<core::Piece> = Vec::new();
        for p in pieces.iter() {
            ps.push((*p as usize).into());
        }
        self.game.piece_gen.append(&ps);
    }
    #[wasm_bindgen(js_name = setupFallingPiece)]
    pub fn setup_falling_piece(&mut self) -> Result<JsValue, JsValue> {
        match self.game.setup_falling_piece(None) {
            Ok(_) => Ok(JsValue::UNDEFINED),
            Err(e) => Err(e.into()),
        }
    }
    pub fn drop(&mut self, n: i8) -> Result<JsValue, JsValue> {
        match self.game.drop(n) {
            Ok(_) => Ok(JsValue::UNDEFINED),
            Err(e) => Err(e.into()),
        }
    }
    #[wasm_bindgen(js_name = firmDrop)]
    pub fn firm_drop(&mut self) -> Result<JsValue, JsValue> {
        match self.game.firm_drop() {
            Ok(_) => Ok(JsValue::UNDEFINED),
            Err(e) => Err(e.into()),
        }
    }
    pub fn shift(&mut self, n: i8, to_end: bool) -> Result<JsValue, JsValue> {
        match self.game.shift(n, to_end) {
            Ok(_) => Ok(JsValue::UNDEFINED),
            Err(e) => Err(e.into()),
        }
    }
    pub fn rotate(&mut self, n: i8) -> Result<JsValue, JsValue> {
        match self.game.rotate(n) {
            Ok(_) => Ok(JsValue::UNDEFINED),
            Err(e) => Err(e.into()),
        }
    }
    pub fn lock(&mut self) -> Result<bool, JsValue> {
        match self.game.lock() {
            Ok(b) => Ok(b),
            Err(e) => Err(e.into()),
        }
    }
    pub fn hold(&mut self) -> Result<bool, JsValue> {
        match self.game.hold() {
            Ok(b) => Ok(b),
            Err(e) => Err(e.into()),
        }
    }
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.game.to_string()
    }
}
