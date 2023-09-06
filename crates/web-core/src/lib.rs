use wasm_bindgen::prelude::*;
use rand::SeedableRng;
use deep_trinity_core::MovePathItem;
use deep_trinity_grid::Grid;

#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    if cfg!(feature = "console_error_panic_hook") {
        console_error_panic_hook::set_once();
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone)]
#[repr(u8)]
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
#[repr(u8)]
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

impl From<deep_trinity_core::Cell> for Cell {
    fn from(c: deep_trinity_core::Cell) -> Self { CELLS[c.to_u8() as usize] }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
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

impl Into<deep_trinity_core::Placement> for Placement {
    fn into(self) -> deep_trinity_core::Placement {
        deep_trinity_core::Placement::new(deep_trinity_core::Orientation::try_from_u8(self.orientation).unwrap(), (self.x, self.y).into())
    }
}

impl From<deep_trinity_core::Placement> for Placement {
    fn from(p: deep_trinity_core::Placement) -> Self {
        Self { orientation: p.orientation.to_u8(), x: p.pos.0, y: p.pos.1 }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub enum Move {
    Right,
    Left,
    Down,
    Cw,
    Ccw,
}

impl Into<deep_trinity_core::Move> for Move {
    fn into(self) -> deep_trinity_core::Move {
        match self {
            Move::Right => deep_trinity_core::Move::Shift(1),
            Move::Left => deep_trinity_core::Move::Shift(-1),
            Move::Down => deep_trinity_core::Move::Drop(1),
            Move::Cw => deep_trinity_core::Move::Rotate(1),
            Move::Ccw => deep_trinity_core::Move::Rotate(-1),
        }
    }
}

impl From<deep_trinity_core::Move> for Move {
    fn from(mv: deep_trinity_core::Move) -> Self {
        match mv {
            deep_trinity_core::Move::Shift(1) => Move::Right,
            deep_trinity_core::Move::Shift(-1) => Move::Left,
            deep_trinity_core::Move::Drop(1) => Move::Down,
            deep_trinity_core::Move::Rotate(1) => Move::Cw,
            deep_trinity_core::Move::Rotate(-1) => Move::Ccw,
            _ => panic!("invalid deep_trinity_core::Move: {:?}", mv),
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug)]
pub struct MoveTransition {
    pub src: Option<Placement>,
    pub by: Option<Move>,
    pub dst: Placement,
}

impl Into<deep_trinity_core::MoveTransition> for MoveTransition {
    fn into(self) -> deep_trinity_core::MoveTransition {
        deep_trinity_core::MoveTransition::new(
            self.dst.into(),
            if let (Some(src), Some(by)) = (self.src, self.by) {
                Some(MovePathItem::new(by.into(), src.into()))
            } else {
                None
            },
        )
    }
}

impl From<deep_trinity_core::MoveTransition> for MoveTransition {
    fn from(mt: deep_trinity_core::MoveTransition) -> Self {
        if let Some(hint) = mt.hint {
            Self { src: Some(hint.placement.into()), by: Some(hint.by.into()), dst: mt.placement.into() }
        } else {
            Self { src: None, by: None, dst: mt.placement.into() }
        }
    }
}

#[wasm_bindgen]
pub struct Game {
    game: deep_trinity_core::Game<'static>,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            game: Default::default(),
        }
    }
    pub fn width(&self) -> deep_trinity_grid::X { self.game.state.playfield.width() }
    pub fn height(&self) -> deep_trinity_grid::Y { self.game.state.playfield.height() }
    #[wasm_bindgen(js_name = visibleHeight)]
    pub fn visible_height(&self) -> deep_trinity_grid::Y { self.game.state.playfield.visible_height }
    #[wasm_bindgen(js_name = getCell)]
    pub fn get_cell(&self, x: i8, y: i8) -> Cell { self.game.get_cell((x, y).into()).into() }
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
    pub fn get_current_num_combos(&self) -> Option<deep_trinity_core::Count> { self.game.state.num_combos }
    #[wasm_bindgen(js_name = getCurrentNumBTBs)]
    pub fn get_current_num_btbs(&self) -> Option<deep_trinity_core::Count> { self.game.state.num_btbs }
    #[wasm_bindgen(js_name = getStatsCount)]
    pub fn get_stats_count(&self, t: StatisticsEntryType) -> deep_trinity_core::Count {
        self.game.stats.get(match t {
            StatisticsEntryType::Single => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(1, None)),
            StatisticsEntryType::Double => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(2, None)),
            StatisticsEntryType::Triple => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(3, None)),
            StatisticsEntryType::Tetris => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(4, None)),
            StatisticsEntryType::Tst => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(3, Some(deep_trinity_core::TSpin::Standard))),
            StatisticsEntryType::Tsd => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(2, Some(deep_trinity_core::TSpin::Standard))),
            StatisticsEntryType::Tss => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(1, Some(deep_trinity_core::TSpin::Standard))),
            StatisticsEntryType::Tsmd => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(2, Some(deep_trinity_core::TSpin::Mini))),
            StatisticsEntryType::Tsms => deep_trinity_core::StatisticsEntryType::LineClear(deep_trinity_core::LineClear::new(1, Some(deep_trinity_core::TSpin::Mini))),
            StatisticsEntryType::MaxCombos => deep_trinity_core::StatisticsEntryType::MaxCombos,
            StatisticsEntryType::MaxBtbs => deep_trinity_core::StatisticsEntryType::MaxBtbs,
            StatisticsEntryType::PerfectClear => deep_trinity_core::StatisticsEntryType::PerfectClear,
            StatisticsEntryType::Hold => deep_trinity_core::StatisticsEntryType::Hold,
            StatisticsEntryType::Lock => deep_trinity_core::StatisticsEntryType::Lock,
        })
    }
    #[wasm_bindgen(js_name = supplyNextPieces)]
    pub fn supply_next_pieces(&mut self, pieces: &[u8]) {
        let mut ps: Vec<deep_trinity_core::Piece> = Vec::new();
        for p in pieces.iter() {
            ps.push(deep_trinity_core::Piece::try_from_u8(*p).unwrap());
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
    gen: deep_trinity_core::RandomPieceGenerator<rand::rngs::StdRng>,
}

#[wasm_bindgen]
impl RandomPieceGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Self {
        Self {
            gen: deep_trinity_core::RandomPieceGenerator::new(rand::rngs::StdRng::seed_from_u64(seed))
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
pub struct Action {
    bot_action: deep_trinity_bot::Action,
}

#[wasm_bindgen]
impl Action {
    fn new(bot_action: deep_trinity_bot::Action) -> Self {
        Self { bot_action }
    }
    pub fn dst(&self) -> Option<MoveTransition> {
        match &self.bot_action {
            deep_trinity_bot::Action::Move(mt) => Some((*mt).into()),
            _ => None,
        }
    }
    #[wasm_bindgen(js_name = isHold)]
    pub fn is_hold(&self) -> bool {
        match self.bot_action {
            deep_trinity_bot::Action::Hold => true,
            _ => false,
        }
    }
}

#[wasm_bindgen]
pub struct Bot {
    bot: Box<dyn deep_trinity_bot::Bot>,
}

#[wasm_bindgen]
impl Bot {
    #[wasm_bindgen(constructor)]
    pub fn new(bot_type: Option<u8>) -> Result<Bot, JsValue> {
        let bot: Box<dyn deep_trinity_bot::Bot> = match bot_type.unwrap_or(1) {
            1 => Box::new(deep_trinity_bot::simple::SimpleBot::default()),
            2 => Box::new(deep_trinity_bot::simple_tree::SimpleTreeBot::default()),
            3 => Box::new(deep_trinity_bot::mcts_puct::MctsPuctBot::default()),
            _ => return Err("invalid bot type".into()),
        };
        Ok(Self { bot })
    }
    pub fn think(&mut self, game: &Game) -> Result<Action, JsValue> {
        match self.bot.think(&game.game) {
            Err(e) => Err(e.to_string().into()),
            Ok(action) => Ok(Action::new(action)),
        }
    }
}

#[wasm_bindgen]
pub struct MovePlayer {
    move_player: deep_trinity_core::MovePlayer,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
impl MovePlayer {
    pub fn from(game: &Game, mt: &MoveTransition) -> Result<MovePlayer, JsValue> {
        let path = game.game.get_almost_good_move_path(&((*mt).into()))?;
        log(&format!("{:?}: {:?}", &game.game.state.falling_piece.as_ref().unwrap().piece_spec.piece, path));
        Ok(Self {
            move_player: deep_trinity_core::MovePlayer::new(path),
        })
    }
    pub fn step(&mut self, game: &mut Game) -> Result<bool, JsValue> {
        self.move_player.step(&mut game.game).map_err(|e| { e.into() })
    }
    #[wasm_bindgen(js_name = isEnd)]
    pub fn is_end(&self) -> bool { self.move_player.is_end() }
}
