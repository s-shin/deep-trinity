use wasm_bindgen::prelude::*;
use rand::SeedableRng;
use deep_trinity_core::prelude::*;

#[wasm_bindgen(js_name = setPanicHook)]
pub fn set_panic_hook() {
    if cfg!(feature = "console_error_panic_hook") {
        console_error_panic_hook::set_once();
    }
}

#[wasm_bindgen(js_name = Piece)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum JsPiece {
    S,
    Z,
    L,
    J,
    I,
    T,
    O,
}

#[wasm_bindgen(js_name = Cell)]
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum JsCell {
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

static CELLS: [JsCell; 10] = [JsCell::Empty, JsCell::Any, JsCell::S, JsCell::Z, JsCell::L, JsCell::J, JsCell::I, JsCell::T, JsCell::O, JsCell::Garbage];

impl From<Cell> for JsCell {
    fn from(c: Cell) -> Self { CELLS[c.to_u8() as usize] }
}

#[wasm_bindgen(js_name = StatisticsEntryType)]
#[derive(Copy, Clone, Debug)]
pub enum JsStatisticsEntryType {
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

#[wasm_bindgen(js_name = Placement)]
#[derive(Copy, Clone, Debug)]
pub struct JsPlacement {
    pub orientation: u8,
    pub x: i8,
    pub y: i8,
}

impl Into<Placement> for JsPlacement {
    fn into(self) -> Placement {
        Placement::new(Orientation::try_from_u8(self.orientation).unwrap(), (self.x, self.y).into())
    }
}

impl From<Placement> for JsPlacement {
    fn from(p: Placement) -> Self {
        Self { orientation: p.orientation.to_u8(), x: p.pos.0, y: p.pos.1 }
    }
}

#[wasm_bindgen(js_name = MoveType)]
#[derive(Copy, Clone, Debug)]
pub enum JsMoveType {
    Right,
    Left,
    Down,
    Cw,
    Ccw,
}

impl Into<Move> for JsMoveType {
    fn into(self) -> Move {
        match self {
            JsMoveType::Right => Move::Shift(1),
            JsMoveType::Left => Move::Shift(-1),
            JsMoveType::Down => Move::Drop(1),
            JsMoveType::Cw => Move::Rotate(1),
            JsMoveType::Ccw => Move::Rotate(-1),
        }
    }
}

impl From<Move> for JsMoveType {
    fn from(mv: Move) -> Self {
        match mv {
            Move::Shift(n) if n > 0 => JsMoveType::Right,
            Move::Shift(n) if n < 0 => JsMoveType::Left,
            Move::Drop(n) if n > 0 => JsMoveType::Down,
            Move::Rotate(n) if n > 0 => JsMoveType::Cw,
            Move::Rotate(n) if n < 0 => JsMoveType::Ccw,
            _ => panic!("invalid deep_trinity_core::Move: {:?}", mv),
        }
    }
}

#[wasm_bindgen(js_name = MoveTransition)]
#[derive(Copy, Clone, Debug)]
pub struct JsMoveTransition {
    pub src: Option<JsPlacement>,
    pub by: Option<JsMoveType>,
    pub dst: JsPlacement,
}

impl Into<MoveTransition> for JsMoveTransition {
    fn into(self) -> MoveTransition {
        MoveTransition::new(
            self.dst.into(),
            if let (Some(src), Some(by)) = (self.src, self.by) {
                Some(MovePathItem::new(by.into(), src.into()))
            } else {
                None
            },
        )
    }
}

impl From<MoveTransition> for JsMoveTransition {
    fn from(mt: MoveTransition) -> Self {
        if let Some(hint) = mt.hint {
            Self { src: Some(hint.placement.into()), by: Some(hint.by.into()), dst: mt.placement.into() }
        } else {
            Self { src: None, by: None, dst: mt.placement.into() }
        }
    }
}

#[wasm_bindgen(js_name = Game)]
pub struct JsGame {
    game: Game<'static>,
}

#[wasm_bindgen]
impl JsGame {
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
    pub fn get_cell(&self, x: i8, y: i8) -> JsCell { self.game.get_cell((x, y).into()).into() }
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
    pub fn get_stats_count(&self, t: JsStatisticsEntryType) -> deep_trinity_core::Count {
        self.game.stats.get(match t {
            JsStatisticsEntryType::Single => StatisticsEntryType::LineClear(LineClear::new(1, None)),
            JsStatisticsEntryType::Double => StatisticsEntryType::LineClear(LineClear::new(2, None)),
            JsStatisticsEntryType::Triple => StatisticsEntryType::LineClear(LineClear::new(3, None)),
            JsStatisticsEntryType::Tetris => StatisticsEntryType::LineClear(LineClear::new(4, None)),
            JsStatisticsEntryType::Tst => StatisticsEntryType::LineClear(LineClear::new(3, Some(TSpin::Standard))),
            JsStatisticsEntryType::Tsd => StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Standard))),
            JsStatisticsEntryType::Tss => StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Standard))),
            JsStatisticsEntryType::Tsmd => StatisticsEntryType::LineClear(LineClear::new(2, Some(TSpin::Mini))),
            JsStatisticsEntryType::Tsms => StatisticsEntryType::LineClear(LineClear::new(1, Some(TSpin::Mini))),
            JsStatisticsEntryType::MaxCombos => StatisticsEntryType::MaxCombos,
            JsStatisticsEntryType::MaxBtbs => StatisticsEntryType::MaxBtbs,
            JsStatisticsEntryType::PerfectClear => StatisticsEntryType::PerfectClear,
            JsStatisticsEntryType::Hold => StatisticsEntryType::Hold,
            JsStatisticsEntryType::Lock => StatisticsEntryType::Lock,
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
    // pub fn valid_moves(&self) -> Result<>
    #[wasm_bindgen(js_name = toString)]
    pub fn to_string(&self) -> String {
        self.game.to_string()
    }
}

#[wasm_bindgen(js_name = RandomPieceGenerator)]
pub struct JsRandomPieceGenerator {
    gen: RandomPieceGenerator<rand::rngs::StdRng>,
}

#[wasm_bindgen]
impl JsRandomPieceGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new(seed: u64) -> Self {
        Self {
            gen: RandomPieceGenerator::new(rand::rngs::StdRng::seed_from_u64(seed))
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

#[wasm_bindgen(js_name = Action)]
pub struct JsAction {
    bot_action: Action,
}

#[wasm_bindgen]
impl JsAction {
    fn new(bot_action: Action) -> Self {
        Self { bot_action }
    }
    pub fn dst(&self) -> Option<JsMoveTransition> {
        match &self.bot_action {
            Action::Move(mt) => Some((*mt).into()),
            _ => None,
        }
    }
    #[wasm_bindgen(js_name = isHold)]
    pub fn is_hold(&self) -> bool {
        match self.bot_action {
            Action::Hold => true,
            _ => false,
        }
    }
}

#[wasm_bindgen(js_name = Bot)]
pub struct JsBot {
    bot: Box<dyn Bot>,
}

#[wasm_bindgen]
impl JsBot {
    #[wasm_bindgen(constructor)]
    pub fn new(bot_type: Option<u8>) -> Result<JsBot, JsValue> {
        let bot: Box<dyn Bot> = match bot_type.unwrap_or(1) {
            1 => Box::new(deep_trinity_bot::simple::SimpleBot::default()),
            // 2 => Box::new(deep_trinity_bot::simple_tree::SimpleTreeBot::default()),
            // 3 => Box::new(deep_trinity_bot::mcts_puct::MctsPuctBot::default()),
            _ => return Err("invalid bot type".into()),
        };
        Ok(Self { bot })
    }
    pub fn think(&mut self, game: &JsGame) -> Result<JsAction, JsValue> {
        match self.bot.think(&game.game) {
            Err(e) => Err(e.to_string().into()),
            Ok(action) => Ok(JsAction::new(action)),
        }
    }
}

#[wasm_bindgen(js_name = MovePlayer)]
pub struct JsMovePlayer {
    move_player: MovePlayer,
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
impl JsMovePlayer {
    pub fn from(game: &JsGame, mt: &JsMoveTransition) -> Result<JsMovePlayer, JsValue> {
        let path = game.game.get_almost_good_move_path(&((*mt).into()))?;
        log(&format!("{:?}: {:?}", &game.game.state.falling_piece.as_ref().unwrap().piece_spec.piece, path));
        Ok(Self {
            move_player: deep_trinity_core::MovePlayer::new(path),
        })
    }
    pub fn step(&mut self, game: &mut JsGame) -> Result<bool, JsValue> {
        self.move_player.step(&mut game.game).map_err(|e| { e.into() })
    }
    #[wasm_bindgen(js_name = isEnd)]
    pub fn is_end(&self) -> bool { self.move_player.is_end() }
}
