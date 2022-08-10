use std::fmt::{Debug, Display, Formatter};
use std::mem;

//--------------------------------------------------------------------------------------------------
// Piece
//--------------------------------------------------------------------------------------------------

const NUM_PIECES: u8 = 7;

#[derive(Copy, Clone, Eq, PartialEq)]
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
    pub fn from_char(c: char) -> Result<Self, &'static str> {
        Ok(match c {
            'L' => Self::L,
            'J' => Self::J,
            'S' => Self::S,
            'Z' => Self::Z,
            'I' => Self::I,
            'T' => Self::T,
            'O' => Self::O,
            _ => return Err("invalid piece char"),
        })
    }
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

impl From<char> for Piece {
    fn from(c: char) -> Self { Self::from_char(c).unwrap() }
}

//--------------------------------------------------------------------------------------------------
// PieceSequence
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone, Default)]
struct PieceSequence(u64);

impl PieceSequence {
    const NUM_BITS: u8 = 64;
    const NUM_PIECE_BITS: u8 = 3;
    const PIECE_MASK: u64 = 0b111;
    pub const MAX_SIZE: u8 = Self::NUM_BITS / Self::NUM_PIECE_BITS;
    fn piece_to_value(p: Piece) -> u8 { p as u8 + 1 }
    fn value_to_piece(v: u8) -> Piece { (v - 1).into() }
    fn last_idx(&self) -> Option<u8> {
        if self.is_empty() {
            None
        } else {
            Some((Self::NUM_BITS - 1 - self.0.leading_zeros() as u8) / Self::NUM_PIECE_BITS)
        }
    }
    pub fn is_empty(&self) -> bool { self.0 == 0 }
    pub fn len(&self) -> u8 { self.last_idx().map(|i| i + 1).unwrap_or(0) }
    pub fn get(&self, i: u8) -> Option<Piece> {
        let v = (self.0 >> (i * Self::NUM_PIECE_BITS)) & Self::PIECE_MASK;
        if v == 0 {
            None
        } else {
            Some(Self::value_to_piece(v as u8))
        }
    }
    pub fn push_back(&mut self, piece: Piece) -> bool {
        if let Some(last_i) = self.last_idx() {
            let i = last_i + 1;
            debug_assert!(i > 0);
            if i == Self::MAX_SIZE {
                return false;
            }
            self.0 |= (Self::piece_to_value(piece) as u64) << (i * Self::NUM_PIECE_BITS);
        } else {
            self.0 |= Self::piece_to_value(piece) as u64;
        }
        true
    }
    pub fn pop_front(&mut self) -> Option<Piece> {
        let p = self.get(0);
        if p.is_some() {
            self.0 = self.0 >> Self::NUM_PIECE_BITS;
        }
        p
    }
    pub fn from_str(s: &str) -> Result<Self, &'static str> {
        let mut seq: Self = Default::default();
        for c in s.chars() {
            seq.push_back(Piece::from_char(c)?);
        }
        Ok(seq)
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

impl From<&str> for PieceSequence {
    fn from(s: &str) -> Self { Self::from_str(s).unwrap() }
}

//--------------------------------------------------------------------------------------------------
// UniquePieceBag
//--------------------------------------------------------------------------------------------------

#[derive(Default, Copy, Clone)]
struct UniquePieceBag(u8);

impl UniquePieceBag {
    pub const EMPTY: UniquePieceBag = UniquePieceBag(0);
    pub const ALL: UniquePieceBag = UniquePieceBag(0b1111111);
    const PIECE_FLAGS: [u8; 7] = [
        0b0000001,
        0b0000010,
        0b0000100,
        0b0001000,
        0b0010000,
        0b0100000,
        0b1000000,
    ];
    pub fn is_empty(&self) -> bool { self.0 == 0 }
    pub fn pop(&mut self) -> Option<Piece> {
        if self.is_empty() {
            None
        } else {
            let idx = self.0.trailing_zeros() as u8;
            self.0 &= self.0 - 1;
            Some(Piece::from(idx % NUM_PIECES))
        }
    }
    fn remove(&mut self, piece: Piece) -> bool {
        let flag = Self::PIECE_FLAGS[piece as usize];
        if self.0 & flag != 0 {
            self.0 &= !flag;
            return true;
        }
        false
    }
}

//--------------------------------------------------------------------------------------------------
// RemainingPiece
//--------------------------------------------------------------------------------------------------

#[derive(Copy, Clone)]
struct RemainingPieces {
    bag1: UniquePieceBag,
    bag2: UniquePieceBag,
}

impl RemainingPieces {
    fn is_empty(&self) -> bool { self.bag1.is_empty() && self.bag2.is_empty() }
    fn pop(&mut self) -> Option<Piece> {
        if self.is_empty() {
            None
        } else {
            self.bag1.pop().or_else(|| self.bag2.pop())
        }
    }
    fn remove(&mut self, piece: Piece) -> bool {
        if self.bag1.remove(piece) {
            return true;
        }
        self.bag2.remove(piece)
    }
}

impl Default for RemainingPieces {
    fn default() -> Self {
        Self {
            bag1: UniquePieceBag::ALL,
            bag2: UniquePieceBag::EMPTY,
        }
    }
}

//--------------------------------------------------------------------------------------------------
// HoldRecorder
//--------------------------------------------------------------------------------------------------

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

//--------------------------------------------------------------------------------------------------
// HoldState
//--------------------------------------------------------------------------------------------------

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

//--------------------------------------------------------------------------------------------------
// enumerate
//--------------------------------------------------------------------------------------------------

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

//--------------------------------------------------------------------------------------------------
// main
//--------------------------------------------------------------------------------------------------

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
        println!("# Hold: {:?}", initial_hold_state.piece);
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

//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn piece_sequence_basic() {
        let mut seq = PieceSequence::default();
        // is_empty, len, push_back
        assert!(seq.is_empty());
        assert_eq!(0, seq.len());
        assert!(seq.push_back(Piece::I));
        assert!(!seq.is_empty());
        assert_eq!(1, seq.len());
        assert!(seq.push_back(Piece::T));
        assert!(seq.push_back(Piece::O));
        assert_eq!(3, seq.len());
        // Display
        assert_eq!("ITO", format!("{}", &seq));
        // get
        assert_eq!(Some(Piece::I), seq.get(0));
        assert_eq!(Some(Piece::T), seq.get(1));
        assert_eq!(Some(Piece::O), seq.get(2));
        // pop_front
        assert_eq!(Some(Piece::I), seq.pop_front());
        assert_eq!(Some(Piece::T), seq.pop_front());
        assert_eq!(Some(Piece::O), seq.pop_front());
        assert_eq!(None, seq.pop_front());
        assert!(seq.is_empty());
    }
}