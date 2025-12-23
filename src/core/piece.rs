#[derive(PartialEq, Eq, Clone, Copy, Default, PartialOrd, Ord)]
pub enum Piece {
    #[default]
    Pawn = 0,
    Knight = 1,
    Bishop = 2,
    Rook = 3,
    Queen = 4,
    King = 5,
}
const ALL_PIECES: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];
const SLIDE: [bool; 6] = [false, false, true, true, true, false];
pub const PAWN_TYPE_VALUE: usize = 0;
pub const KNIGHT_TYPE_VALUE: usize = 1;
pub const BISHOP_TYPE_VALUE: usize = 2;
pub const ROOK_TYPE_VALUE: usize = 3;
pub const QUEEN_TYPE_VALUE: usize = 4;
pub const KING_TYPE_VALUE: usize = 5;
impl Piece {
    pub fn from_index(i: usize) -> Piece {
        return ALL_PIECES[i];
    }
    pub fn is_sliding(self) -> bool {
        return SLIDE[self as usize];
    }
}
