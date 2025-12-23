use crate::core::square::Square;

pub const FULL_BITBOARD: BitBoard = !0u64;
#[allow(unused)]
pub const EMPTY_BITBOARD: BitBoard = 0u64;
#[allow(unused)]
pub const FILE_BITBOARDS: [BitBoard; 8] = [
    0x101010101010101,
    0x202020202020202,
    0x404040404040404,
    0x808080808080808,
    0x1010101010101010,
    0x2020202020202020,
    0x4040404040404040,
    0x8080808080808080,
];
pub const RANK_BITBOARDS: [BitBoard; 8] = [
    0xff,
    0xff00,
    0xff0000,
    0xff000000,
    0xff00000000,
    0xff0000000000,
    0xff000000000000,
    0xff00000000000000,
];
#[allow(unused)]
pub const SEVENTH_RANK: u64 = RANK_BITBOARDS[6];
#[allow(unused)]
pub const SECOND_RANK: u64 = RANK_BITBOARDS[1];
#[allow(unused)]
pub const DARK_SQUARES: u64 = 0xAA55AA55AA55AA55;
#[allow(unused)]
pub const LIGHT_SQUARES: u64 = 0x55AA55AA55AA55AA;
pub trait BitBoardUtil<T> {
    #[allow(unused)]
    fn print(&self);
    fn pop_lsb(&mut self) -> Square;
    fn has_sq(&self, sq: Square) -> bool;
    fn to_sq(&self) -> Square;
}
pub type BitBoard = u64;

impl BitBoardUtil<BitBoard> for BitBoard {
    fn print(&self) {
        for rank in (0..8).rev() {
            let sq = rank * 8;
            for i in 0..8 {
                if self & (1 << (sq + i)) != 0 {
                    print!("1");
                } else {
                    print!("0");
                }
            }
            println!();
        }
        println!();
    }
    fn pop_lsb(&mut self) -> Square {
        let x = self.trailing_zeros() as Square;
        *self &= *self - 1;
        return x;
    }
    fn has_sq(&self, sq: Square) -> bool {
        return (self & (1 << sq)) != 0;
    }
    fn to_sq(&self) -> Square {
        return self.trailing_zeros() as Square;
    }
}
