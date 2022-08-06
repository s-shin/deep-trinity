use std::fmt::{Debug, Display, Formatter};
use std::mem;

const NUM_PIECES: u8 = 7;

#[derive(Copy, Clone)]
#[repr(u8)]
enum Piece {
    L,
    J,
    S,
    Z,
    I,
    T,
    O,
}

const PIECE_CHARS: [char; 7] = ['L', 'J', 'S', 'Z', 'I', 'T', 'O'];

impl Piece {
    pub fn to_char(&self) -> char { PIECE_CHARS[*self as usize] }
}

impl Display for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl Debug for Piece {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl From<u8> for Piece {
    fn from(v: u8) -> Self {
        assert!(v < NUM_PIECES);
        unsafe { mem::transmute(v) }
    }
}

#[derive(Copy, Clone, Default)]
struct PieceSequence(u64);

impl PieceSequence {
    const NUM_BITS: u8 = 64;
    const MAX_COUNT: u8 = 8;
    const PIECE_FLAGS: [u64; 7] = [
        0b0000001,
        0b0000010,
        0b0000100,
        0b0001000,
        0b0010000,
        0b0100000,
        0b1000000,
    ];

    pub fn len(&self) -> u8 { self.last_idx().map(|i| i + 1).unwrap_or(0) }
    pub fn is_empty(&self) -> bool { self.0 == 0 }
    fn first_idx(&self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            Some(self.0.trailing_zeros() as u8 / NUM_PIECES)
        }
    }
    fn last_idx(&self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            Some((Self::NUM_BITS - 1 - self.0.leading_zeros() as u8) / NUM_PIECES)
        }
    }
    pub fn push_back(&mut self, piece: Piece) -> bool {
        if let Some(last_i) = self.last_idx() {
            let i = last_i + 1;
            if i == Self::MAX_COUNT {
                return false;
            }
            self.0 |= Self::PIECE_FLAGS[piece as usize] << (i * NUM_PIECES);
        } else {
            self.0 |= Self::PIECE_FLAGS[piece as usize];
        }
        true
    }
    pub fn pop_front(&mut self) -> Option<Piece> {
        if let Some(i) = self.first_idx() {
            let p = Piece::from((self.0 >> (i * NUM_PIECES)).trailing_zeros() as u8);
            self.0 &= self.0 - 1;
            Some(p)
        } else {
            None
        }
    }
}

impl Display for PieceSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut seq = self.clone();
        while let Some(p) = seq.pop_front() {
            write!(f, "{}", p)?;
        }
        Ok(())
    }
}

impl Debug for PieceSequence {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone)]
struct RemainingPieces(u16);

impl RemainingPieces {
    const FIRST_BAG_ALL: u16 = 0b1111111;
    const FIRST_BAG: [u16; 7] = [
        0b00000000000001,
        0b00000000000010,
        0b00000000000100,
        0b00000000001000,
        0b00000000010000,
        0b00000000100000,
        0b00000001000000,
    ];
    const SECOND_BAG: [u16; 7] = [
        0b00000010000000,
        0b00000100000000,
        0b00001000000000,
        0b00010000000000,
        0b00100000000000,
        0b01000000000000,
        0b10000000000000,
    ];

    fn is_empty(&self) -> bool { self.0 == 0 }
    fn pop(&mut self) -> Option<Piece> {
        if self.is_empty() {
            None
        } else {
            let idx = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1;
            Some(Piece::from(idx % NUM_PIECES))
        }
    }
    fn flags(&self, piece: Piece) -> [u16; 2] { [Self::FIRST_BAG[piece as usize], Self::SECOND_BAG[piece as usize]] }
    fn remove(&mut self, piece: Piece) -> bool {
        for flag in self.flags(piece).iter() {
            if self.0 & flag != 0 {
                self.0 &= !flag;
                return true;
            }
        }
        false
    }
}

impl Default for RemainingPieces {
    fn default() -> Self { Self(Self::FIRST_BAG_ALL) }
}

#[derive(Copy, Clone)]
struct HoldRecorder(u8);

impl HoldRecorder {
    const MAX_ENTRIES: u8 = 7;

    fn new(can_hold_first: bool) -> Self {
        if can_hold_first {
            Self(0)
        } else {
            Self(1)
        }
    }
    fn is_held(&self, nth: u8) -> bool {
        assert!(0 < nth && nth <= Self::MAX_ENTRIES);
        self.0 & (1 << (nth - 1)) != 0
    }
    fn hold(&mut self, nth: u8) {
        assert!(!self.is_held(nth));
        self.0 |= 1 << (nth - 1);
    }
    #[allow(dead_code)]
    fn pop(&mut self) -> Option<u8> {
        if self.0 > 0 {
            let nth = self.0.trailing_zeros() + 1;
            self.0 &= self.0 - 1;
            Some(nth as u8)
        } else {
            None
        }
    }
}

impl Display for HoldRecorder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:07b}", (self.0 << 1).reverse_bits())
    }
}

impl Debug for HoldRecorder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Copy, Clone)]
struct HoldState {
    piece: Option<Piece>,
    recorder: HoldRecorder,
}

impl HoldState {
    fn new(piece: Option<Piece>, recorder: HoldRecorder) -> Self {
        Self { piece, recorder }
    }
}

impl Display for HoldState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}:{}",
            if let Some(p) = self.piece { p.to_char() } else { '_' },
            self.recorder,
        )
    }
}

fn enumerate_internal(remains: RemainingPieces, consumed: PieceSequence, hold_state: HoldState, cb: &mut impl FnMut(PieceSequence, HoldState) -> ()) {
    if remains.is_empty() {
        cb(consumed, hold_state);
        return;
    }
    let nth_consumption = consumed.len() + 1;
    let mut remains_iter = remains.clone();
    while let Some(p) = remains_iter.pop() {
        {
            let mut next_remains = remains.clone();
            next_remains.remove(p);
            let mut next_consumed = consumed.clone();
            let ok = next_consumed.push_back(p);
            debug_assert!(ok);
            enumerate_internal(next_remains, next_consumed, hold_state, cb);
        }
        if !hold_state.recorder.is_held(nth_consumption) {
            let mut next_remains = remains.clone();
            next_remains.remove(p);
            let mut recorder = hold_state.recorder.clone();
            recorder.hold(nth_consumption);
            let next_hold_state = HoldState::new(Some(p), recorder);
            if let Some(hold_piece) = hold_state.piece {
                // consume popped piece
                let mut next_consumed = consumed.clone();
                let ok = next_consumed.push_back(hold_piece);
                debug_assert!(ok);
                enumerate_internal(next_remains, next_consumed, next_hold_state, cb);
            } else {
                // consume no piece
                enumerate_internal(next_remains, consumed, next_hold_state, cb);
            }
        }
    }
}

fn enumerate(hold_state: HoldState, cb: &mut impl FnMut(PieceSequence, HoldState) -> ()) {
    enumerate_internal(Default::default(), Default::default(), hold_state, cb);
}

fn main() {
    use Piece::*;

    let mut initial_hold_states = vec![
        HoldState::new(None, HoldRecorder::new(true)),
    ];
    for piece in [L, J, S, Z, I, T, O] {
        initial_hold_states.push(HoldState::new(Some(piece), HoldRecorder::new(true)));
    }

    for initial_hold_state in initial_hold_states {
        let mut i = 0;
        println!("# {:?}", initial_hold_state.piece);
        enumerate(initial_hold_state, &mut |pattern, hold_state| {
            i += 1;
            const INTERVAL: i32 = 100000;
            if i % INTERVAL == 0 {
                println!("{}... ({} {})", i, pattern, hold_state);
            }
        });
        println!("{} patterns found.", i);
    }
}
