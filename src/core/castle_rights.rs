use crate::core::{color::Color, square::Square};
pub trait CastleRightsUtil<T> {
    fn has_king_side(&self, color: Color) -> bool;
    fn has_queen_side(&self, color: Color) -> bool;
    #[allow(unused)]
    fn color(&self, color: Color) -> Rights;
}
pub type CastleRights = u8;
const WHITE_KING_SIDE: CastleRights = 1;
const WHITE_QUEEN_SIDE: CastleRights = 2;
const WHITE_ALL: CastleRights = 3;

const BLACK_KING_SIDE: CastleRights = 4;
const BLACK_QUEEN_SIDE: CastleRights = 8;
const BLACK_ALL: CastleRights = 12;

pub const WHITE_KING_SIDE_REMOVED: CastleRights = !WHITE_KING_SIDE;
pub const WHITE_QUEEN_SIDE_REMOVED: CastleRights = !WHITE_QUEEN_SIDE;
pub const BLACK_KING_SIDE_REMOVED: CastleRights = !BLACK_KING_SIDE;
pub const BLACK_QUEEN_SIDE_REMOVED: CastleRights = !BLACK_QUEEN_SIDE;
pub const WHITE_ALL_REMOVED: CastleRights = !(WHITE_KING_SIDE | WHITE_QUEEN_SIDE);
pub const BLACK_ALL_REMOVED: CastleRights = !(BLACK_KING_SIDE | BLACK_QUEEN_SIDE);

pub const WHITE_KING_SIDE_ROOK: Square = 7;
pub const WHITE_QUEEN_SIDE_ROOK: Square = 0;
pub const BLACK_KING_SIDE_ROOK: Square = 63;
pub const BLACK_QUEEN_SIDE_ROOK: Square = 56;

pub const KING_SIDE_CASTLE_ROOKS: [Square; 2] = [WHITE_KING_SIDE_ROOK, BLACK_KING_SIDE_ROOK];
pub const QUEEN_SIDE_CASTLE_ROOKS: [Square; 2] = [WHITE_QUEEN_SIDE_ROOK, BLACK_QUEEN_SIDE_ROOK];
pub const KING_SIDE_CASTLE_ROOK_DEST: [Square; 2] = [5, 61];
pub const QUEEN_SIDE_CASTLE_ROOK_DEST: [Square; 2] = [3, 59];

impl CastleRightsUtil<CastleRights> for CastleRights {
    fn has_king_side(&self, color: Color) -> bool {
        return (color == Color::White && (self & WHITE_KING_SIDE) != 0)
            || (color == Color::Black && (self & BLACK_KING_SIDE) != 0);
    }
    fn has_queen_side(&self, color: Color) -> bool {
        return (color == Color::White && (self & WHITE_QUEEN_SIDE) != 0)
            || (color == Color::Black && (self & BLACK_QUEEN_SIDE) != 0);
    }
    fn color(&self, color: Color) -> Rights {
        let x = if color == Color::White {
            self & WHITE_ALL
        } else {
            (self & BLACK_ALL) >> 2
        };
        match x {
            1 => Rights::KingSide,
            2 => Rights::QueenSide,
            3 => Rights::Both,
            _ => Rights::NoRights,
        }
    }
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Rights {
    NoRights,
    KingSide,
    QueenSide,
    Both,
}
