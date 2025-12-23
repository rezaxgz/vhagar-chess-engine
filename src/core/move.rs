use crate::core::{
    piece::Piece,
    square::{str_to_square, Square, SquareUtil},
};
pub type Move = u16;

const FROM_MASK: Move = 0b111111;
const TO_MASK: Move = 0b111111000000;
const SP_MASK: Move = !(FROM_MASK | TO_MASK);

pub const KNIGHT_PROMOTION: u16 = 1;
#[allow(unused)]
pub const BISHOP_PROMOTION: u16 = 2;
#[allow(unused)]
pub const ROOK_PROMOTION: u16 = 3;
pub const QUEEN_PROMOTION: u16 = 4;
pub const KING_SIDE_CASTLE: u16 = 5;
pub const QUEEN_SIDE_CASTLE: u16 = 6;
pub const EN_PASSANT: u16 = 7;

#[derive(PartialEq, Clone, Copy)]
#[allow(unused)]
pub enum SpecialFalg {
    None,
    KnighPromotion,
    BishopPromotion,
    RookPromotion,
    QueenPromotion,
    KingSideCastle,
    QueenSideCastle,
    EnPessant,
}

impl SpecialFalg {
    #[allow(unused)]
    pub fn from_u16(sp: u16) -> Self {
        return match sp {
            1 => Self::KnighPromotion,
            2 => Self::BishopPromotion,
            3 => Self::RookPromotion,
            4 => Self::QueenPromotion,
            5 => Self::KingSideCastle,
            6 => Self::QueenSideCastle,
            7 => Self::EnPessant,
            _ => Self::None,
        };
    }
}
pub const KING_SIDE_CASTLE_MOVES: [Move; 2] = [20868, 24508];
pub const QUEEN_SIDE_CASTLE_MOVES: [Move; 2] = [24708, 28348];

pub const PROMOTION_PIECES: [Piece; 5] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
];
const SP: [&str; 16] = [
    "", "n", "b", "r", "q", "", "", "", "", "", "", "", "", "", "", "",
];
pub trait MoveUtil<T> {
    fn get_from(&self) -> Square;
    fn get_to(&self) -> Square;
    fn get_sp(&self) -> u16;
    fn to_str(&self) -> String;
    fn is_promotion(&self) -> bool;
    fn is_castle(&self) -> bool;
    fn is_ep(&self) -> bool;
}
impl MoveUtil<Move> for Move {
    fn get_from(&self) -> Square {
        return (self & FROM_MASK) as Square;
    }
    fn get_to(&self) -> Square {
        return ((self & TO_MASK) >> 6) as Square;
    }
    fn get_sp(&self) -> u16 {
        return (self & SP_MASK) >> 12;
    }
    fn to_str(&self) -> String {
        return format!(
            "{}{}{}",
            self.get_from().to_str(),
            self.get_to().to_str(),
            SP[self.get_sp() as usize]
        );
    }
    fn is_promotion(&self) -> bool {
        let sp = self.get_sp();
        return sp > 0 && sp <= QUEEN_PROMOTION;
    }
    fn is_castle(&self) -> bool {
        let sp = self.get_sp();
        return sp == KING_SIDE_CASTLE || sp == QUEEN_SIDE_CASTLE;
    }
    fn is_ep(&self) -> bool {
        return self.get_sp() == EN_PASSANT;
    }
}
pub fn new_move(from: Square, to: Square, sp: u16) -> Move {
    return (from as u16) | ((to as u16) << 6) | (sp << 12);
}
pub fn move_from_string(m: &str) -> Move {
    let c = m.chars().collect::<Vec<char>>();
    if c.len() == 4 {
        return new_move(
            str_to_square(format!("{}{}", c[0], c[1]).as_str()),
            str_to_square(format!("{}{}", c[2], c[3]).as_str()),
            0,
        );
    } else {
        return new_move(
            str_to_square(format!("{}{}", c[0], c[1]).as_str()),
            str_to_square(format!("{}{}", c[2], c[3]).as_str()),
            SP.iter()
                .position(|&r| r == c[4].to_string().as_str())
                .unwrap() as u16,
        );
    }
}
#[derive(PartialEq, Clone, Copy)]
pub struct ExtendedMove {
    pub piece: Piece,
    pub from: Square,
    pub to: Square,
    pub special_falg: SpecialFalg,
}
impl ExtendedMove {
    #[allow(unused)]
    pub fn default() -> Self {
        return ExtendedMove {
            piece: Piece::Pawn,
            from: 0,
            to: 0,
            special_falg: SpecialFalg::None,
        };
    }
}
