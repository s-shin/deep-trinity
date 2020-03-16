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

// impl From<Cell> for core::Cell {
//     fn from(p: Piece) -> Self { (p as usize).into() }
// }

#[wasm_bindgen]
pub struct Game {
    game: core::Game<core::StaticPieceGenerator>,
}

#[wasm_bindgen()]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            game: core::Game::new(Default::default()),
        }
    }
    #[wasm_bindgen]
    pub fn width(&self) -> core::SizeX { self.game.state.playfield.width() }
    #[wasm_bindgen]
    pub fn height(&self) -> core::SizeY { self.game.state.playfield.height() }
    #[wasm_bindgen]
    pub fn visible_height(&self) -> core::SizeY { self.game.state.playfield.visible_height }
    #[wasm_bindgen(js_name = getCell)]
    pub fn get_cell(&self, x: u8, y: u8) -> Cell {
        self.game.get_cell((x, y).into()).into()
    }
    #[wasm_bindgen(js_name = appendNextPiece)]
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
    pub fn to_string(&mut self) -> String {
        self.game.to_string()
    }
}
