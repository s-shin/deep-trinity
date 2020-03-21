extern crate wasm_bindgen;
extern crate console_error_panic_hook;
extern crate rand;
extern crate core;
extern crate bot;

use wasm_bindgen::prelude::*;
use rand::SeedableRng;
use bot::Bot;

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
    Empty,
    Any,
    S,
    Z,
    L,
    J,
    I,
    T,
    O,
    Garbage,
}

static CELLS: [Cell; 10] = [Cell::Empty, Cell::Any, Cell::S, Cell::Z, Cell::L, Cell::J, Cell::I, Cell::T, Cell::O, Cell::Garbage];

impl From<core::Cell> for Cell {
    fn from(c: core::Cell) -> Self { CELLS[core::CellTypeId::from(c).0 as usize] }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
pub enum StatisticsEntryType {
    Single,
    Double,
    Triple,
    Tetris,
    Tst,
    Tsd,
    Tss,
    Tsmd,
    Tsms,
    MaxCombos,
    MaxBtbs,
    PerfectClear,
    Hold,
    Lock,
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct Placement {
    pub orientation: u8,
    pub x: i8,
    pub y: i8,
}

impl Into<core::Placement> for Placement {
    fn into(self) -> core::Placement {
        core::Placement::new(core::Orientation::new(self.orientation), core::Pos(self.x, self.y))
    }
}

impl From<core::Placement> for Placement {
    fn from(p: core::Placement) -> Self {
        Self { orientation: p.orientation.id(), x: p.pos.0, y: p.pos.1 }
    }
}

#[wasm_bindgen]
pub struct Game {
    game: core::Game,
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
    pub fn get_next_pieces(&self, visible: bool) -> Box<[u8]> {
        let np = &self.game.state.next_pieces;
        np.iter()
            .take(if visible { np.visible_num } else { np.len() })
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
            StatisticsEntryType::Single => core::StatisticsEntryType::LineClear(core::LineClear::new(1, None)),
            StatisticsEntryType::Double => core::StatisticsEntryType::LineClear(core::LineClear::new(2, None)),
            StatisticsEntryType::Triple => core::StatisticsEntryType::LineClear(core::LineClear::new(3, None)),
            StatisticsEntryType::Tetris => core::StatisticsEntryType::LineClear(core::LineClear::new(4, None)),
            StatisticsEntryType::Tst => core::StatisticsEntryType::LineClear(core::LineClear::new(3, Some(core::TSpin::Standard))),
            StatisticsEntryType::Tsd => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Standard))),
            StatisticsEntryType::Tss => core::StatisticsEntryType::LineClear(core::LineClear::new(1, Some(core::TSpin::Standard))),
            StatisticsEntryType::Tsmd => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Mini))),
            StatisticsEntryType::Tsms => core::StatisticsEntryType::LineClear(core::LineClear::new(2, Some(core::TSpin::Mini))),
            StatisticsEntryType::MaxCombos => core::StatisticsEntryType::MaxCombos,
            StatisticsEntryType::MaxBtbs => core::StatisticsEntryType::MaxBtbs,
            StatisticsEntryType::PerfectClear => core::StatisticsEntryType::PerfectClear,
            StatisticsEntryType::Hold => core::StatisticsEntryType::Hold,
            StatisticsEntryType::Lock => core::StatisticsEntryType::Lock,
        })
    }
    #[wasm_bindgen(js_name = supplyNextPieces)]
    pub fn supply_next_pieces(&mut self, pieces: &[u8]) {
        let mut ps: Vec<core::Piece> = Vec::new();
        for p in pieces.iter() {
            ps.push((*p as usize).into());
        }
        self.game.supply_next_pieces(&ps);
    }
    #[wasm_bindgen(js_name = shouldSupplyNextPieces)]
    pub fn should_supply_next_pieces(&self) -> bool { self.game.should_supply_next_pieces() }
    #[wasm_bindgen(js_name = isGameOver)]
    pub fn is_game_over(&self) -> bool { self.game.state.is_game_over() }
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

#[wasm_bindgen]
pub struct RandomPieceGenerator {
    gen: core::RandomPieceGenerator<rand::rngs::StdRng>,
}

#[wasm_bindgen]
impl RandomPieceGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Self {
        Self {
            gen: core::RandomPieceGenerator::new(rand::rngs::StdRng::seed_from_u64(seed))
        }
    }
    pub fn generate(&mut self) -> Box<[u8]> {
        self.gen.generate()
            .iter()
            .map(|p| { (*p as usize) as u8 })
            .collect::<Vec<u8>>()
            .into_boxed_slice()
    }
}


#[wasm_bindgen]
#[derive(Default)]
pub struct SimpleBot {
    bot: bot::SimpleBot,
}

#[wasm_bindgen]
impl SimpleBot {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self { Default::default() }
    pub fn think(&mut self, game: &Game) -> Option<Placement> {
        self.bot.think(&game.game).map(|p| { p.into() })
    }
}

#[wasm_bindgen]
pub struct MovePlayer {
    move_player: core::MovePlayer,
}

#[wasm_bindgen]
impl MovePlayer {
    pub fn from(game: &Game, dst: Placement) -> Result<MovePlayer, JsValue> {
        use core::move_search::humanly_optimized::HumanlyOptimizedMoveSearcher;
        use core::move_search::astar::AStarMoveSearcher;

        let g = &game.game;
        for i in 1..2 {
            // let mut searcher = core::move_search::bruteforce::BruteForceMoveSearcher::default();
            let r = match match i {
                0 => g.search_moves(&mut HumanlyOptimizedMoveSearcher::new(dst.into(), true, true)),
                1 => g.search_moves(&mut AStarMoveSearcher::new(dst.into(), false)),
                _ => panic!(),
            } {
                Ok(r) => r,
                Err(e) => { return Err(e.into()); }
            };
            if let Some(rec) = r.get(&dst.into()) {
                return Ok(Self {
                    move_player: core::MovePlayer::new(rec),
                });
            }
        }
        Err("cannot move to".into())
    }
    pub fn step(&mut self, game: &mut Game) -> Result<bool, JsValue> {
        self.move_player.step(&mut game.game).map_err(|e| { e.into() })
    }
    #[wasm_bindgen(js_name = isEnd)]
    pub fn is_end(&self) -> bool { self.move_player.is_end() }
}
