use crate::core::bitboard::{BitBoard, BitBoardUtil};
use crate::core::castle_rights::{
    CastleRights, BLACK_ALL_REMOVED, BLACK_KING_SIDE_REMOVED, BLACK_KING_SIDE_ROOK,
    BLACK_QUEEN_SIDE_REMOVED, BLACK_QUEEN_SIDE_ROOK, KING_SIDE_CASTLE_ROOKS,
    KING_SIDE_CASTLE_ROOK_DEST, QUEEN_SIDE_CASTLE_ROOKS, QUEEN_SIDE_CASTLE_ROOK_DEST,
    WHITE_ALL_REMOVED, WHITE_KING_SIDE_REMOVED, WHITE_KING_SIDE_ROOK, WHITE_QUEEN_SIDE_REMOVED,
    WHITE_QUEEN_SIDE_ROOK,
};
use crate::core::color::Color;
use crate::core::piece::{
    Piece, BISHOP_TYPE_VALUE, KING_TYPE_VALUE, KNIGHT_TYPE_VALUE, PAWN_TYPE_VALUE,
    QUEEN_TYPE_VALUE, ROOK_TYPE_VALUE,
};
use crate::core::r#move::{
    move_from_string, ExtendedMove, Move, MoveUtil, SpecialFalg, EN_PASSANT, KING_SIDE_CASTLE,
    KING_SIDE_CASTLE_MOVES, PROMOTION_PIECES, QUEEN_SIDE_CASTLE, QUEEN_SIDE_CASTLE_MOVES,
};
use crate::core::square::{str_to_square, Square};
use crate::core::tables::magics::{
    get_between, get_bishop_moves, get_bishop_rays, get_king_moves, get_knight_moves,
    get_pawn_attacks, get_pawn_controlled_bb, get_rook_moves, get_rook_rays, EP_TARGETS,
};
use crate::core::tables::zobrist::{
    get_castle_zobrist, get_ep_zobrist, get_piece_zobrist, get_turn_zobrist,
};
use crate::evaluation::tables::get_pst_value;
use crate::search::defs::Score;
// use crate::evaluation::tables::get_pst_value;
const PIECE_LETTERS: [char; 12] = ['p', 'n', 'b', 'r', 'q', 'k', 'P', 'N', 'B', 'R', 'Q', 'K'];
const CASTLE_RIGHTS_LETTERS: [char; 4] = ['K', 'Q', 'k', 'q'];
#[derive(Clone, Copy)]
pub struct Board {
    pub pieces: [BitBoard; 6],
    pub color_combined: [BitBoard; 2],
    pub combined: BitBoard,
    pub checkers: BitBoard,
    pub pinned: BitBoard,
    pub castle_rights: CastleRights,
    pub turn: Color,
    pub hash: u64,
    pub pawn_hash: u64,
    pub en_passant: Option<Square>,
    pub halfmove_clock: u8,
    pub pst_value: Score,
}
impl Board {
    pub fn default() -> Board {
        return Board::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    }
    pub fn from_fen(fen: &str) -> Board {
        let args = fen.split(" ").collect::<Vec<&str>>();
        let ranks = args[0].split("/").collect::<Vec<&str>>();
        let mut board = Board {
            pieces: [0; 6],
            color_combined: [0; 2],
            combined: 0,
            checkers: 0,
            pinned: 0,
            castle_rights: 0,
            turn: Color::White,
            hash: 0,
            en_passant: None,
            pawn_hash: 0,
            halfmove_clock: 0,
            pst_value: 0,
        };
        for i in 0..8 {
            let rank = ranks[7 - i];
            let mut sq = i * 8;
            for c in rank.chars() {
                if c.is_numeric() {
                    sq += c.to_digit(10).unwrap() as usize;
                    continue;
                }
                for n in 0..12 {
                    if PIECE_LETTERS[n] == c {
                        let color = if n > 5 { Color::White } else { Color::Black };
                        board.put_piece(Piece::from_index(n % 6), sq as Square, color);
                    }
                }
                sq += 1;
            }
        }
        for c in args[2].chars() {
            for i in 0..4 {
                if c == CASTLE_RIGHTS_LETTERS[i] {
                    board.castle_rights |= 1 << i;
                }
            }
        }
        board.hash ^= get_castle_zobrist(board.castle_rights);
        if args[1] == "b" {
            board.turn = Color::Black;
            board.hash ^= get_turn_zobrist();
        }
        if args[3] != "-" {
            board.en_passant = Some(str_to_square(args[3]));
            board.hash ^= get_ep_zobrist(board.turn, board.en_passant);
        }
        board.update_pins_and_checks();
        return board;
    }
    pub fn print(&self) {
        for rank in (0..8).rev() {
            let mut chars = [' '; 8];
            for file in 0..8 {
                let sq = rank * 8 + file;
                let piece = self.piece_on(sq);
                if piece.is_some() {
                    let p = if self.color_combined[Color::White as usize].has_sq(sq) {
                        piece.unwrap() as usize + 6
                    } else {
                        piece.unwrap() as usize
                    };
                    chars[file as usize] = PIECE_LETTERS[p];
                }
            }
            println!("{:?}", chars);
        }
    }
    pub fn get_piece_bitboard(&self, piece: Piece, color: Color) -> BitBoard {
        return self.pieces[piece as usize] & self.color_combined[color as usize];
    }
    pub fn king_square(&self, color: Color) -> Square {
        return self.get_piece_bitboard(Piece::King, color).to_sq();
    }
    pub fn get_enemy_pieces(&self) -> BitBoard {
        return self.color_combined[self.turn as usize ^ 1];
    }
    pub fn get_friendly_pieces(&self) -> BitBoard {
        return self.color_combined[self.turn as usize];
    }
    pub fn piece_on(&self, sq: Square) -> Option<Piece> {
        if self.pieces[PAWN_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::Pawn);
        } else if self.pieces[KNIGHT_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::Knight);
        } else if self.pieces[BISHOP_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::Bishop);
        } else if self.pieces[ROOK_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::Rook);
        } else if self.pieces[QUEEN_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::Queen);
        } else if self.pieces[KING_TYPE_VALUE].has_sq(sq) {
            return Some(Piece::King);
        }
        return None;
    }
    pub fn get_line_attackers(&self, color: Color) -> BitBoard {
        return (self.pieces[Piece::Rook as usize] | self.pieces[Piece::Queen as usize])
            & self.color_combined[color as usize];
    }
    pub fn is_controlled(&self, sq: Square, color: Color) -> bool {
        let sq_bb = 1u64 << sq;
        if (get_pawn_controlled_bb(self.get_piece_bitboard(Piece::Pawn, color), color) & sq_bb) != 0
        {
            return true;
        }
        let mut line_attackers = (self.pieces[Piece::Rook as usize]
            | self.pieces[Piece::Queen as usize])
            & self.color_combined[color as usize];
        while line_attackers != 0 {
            if (get_rook_moves(line_attackers.pop_lsb(), self.combined) & sq_bb) != 0 {
                return true;
            }
        }
        let mut diagonal_attackers = (self.pieces[Piece::Bishop as usize]
            | self.pieces[Piece::Queen as usize])
            & self.color_combined[color as usize];
        while diagonal_attackers != 0 {
            if (get_bishop_moves(diagonal_attackers.pop_lsb(), self.combined) & sq_bb) != 0 {
                return true;
            }
        }
        let mut knights = self.get_piece_bitboard(Piece::Knight, color);
        while knights != 0 {
            if (get_knight_moves(knights.pop_lsb()) & sq_bb) != 0 {
                return true;
            }
        }
        return (get_king_moves(self.get_piece_bitboard(Piece::King, color).pop_lsb()) & sq_bb)
            != 0;
    }
    fn update_pins_and_checks(&mut self) {
        self.pinned = 0;
        self.checkers = 0;
        let king = self.get_piece_bitboard(Piece::King, self.turn).to_sq();
        let friendly_pieces = self.get_friendly_pieces();
        let mut diagonal_pinners = (self.pieces[Piece::Bishop as usize]
            | self.pieces[Piece::Queen as usize])
            & self.get_enemy_pieces()
            & get_bishop_rays(king);
        let mut line_pinners = (self.pieces[Piece::Rook as usize]
            | self.pieces[Piece::Queen as usize])
            & self.get_enemy_pieces()
            & get_rook_rays(king);
        while diagonal_pinners != 0 {
            let piece = diagonal_pinners.pop_lsb();
            let bb = get_between(king, piece);
            if (bb & friendly_pieces) != (bb & self.combined) {
                continue;
            }
            match (friendly_pieces & bb).count_ones() {
                0 => {
                    self.checkers |= 1 << piece;
                }
                1 => {
                    self.pinned |= friendly_pieces & bb;
                }
                _ => {}
            }
        }
        while line_pinners != 0 {
            let piece = line_pinners.pop_lsb();
            let bb = get_between(king, piece);
            if (bb & friendly_pieces) != (bb & self.combined) {
                continue;
            }
            match (friendly_pieces & bb).count_ones() {
                0 => {
                    self.checkers |= 1 << piece;
                }
                1 => {
                    self.pinned |= friendly_pieces & bb;
                }
                _ => {}
            }
        }
        self.checkers |=
            self.get_piece_bitboard(Piece::Knight, !self.turn) & get_knight_moves(king);
            
        self.checkers |=
            self.get_piece_bitboard(Piece::Pawn, !self.turn) & get_pawn_attacks(king, self.turn);
    }
    fn put_piece(&mut self, piece: Piece, sq: Square, color: Color) {
        let bb = 1u64 << sq;
        self.pieces[piece as usize] |= bb;
        self.color_combined[color as usize] |= bb;
        self.combined |= bb;
        self.hash ^= get_piece_zobrist(piece, color, sq);
        if piece == Piece::Pawn {
            self.pawn_hash ^= get_piece_zobrist(Piece::Pawn, color, sq);
        } 
        else if piece != Piece::Pawn && piece != Piece::King {
            self.pst_value += get_pst_value(color, piece, sq);
        }
    }
    fn remove_piece(&mut self, piece: Piece, sq: Square, color: Color) {
        let bb = 1u64 << sq;
        self.pieces[piece as usize] ^= bb;
        self.color_combined[color as usize] ^= bb;
        self.combined ^= bb;
        self.hash ^= get_piece_zobrist(piece, color, sq);
        if piece == Piece::Pawn {
            self.pawn_hash ^= get_piece_zobrist(Piece::Pawn, color, sq);
        } 
        else if piece != Piece::Pawn && piece != Piece::King {
            self.pst_value -= get_pst_value(color, piece, sq);
        }
    }
    fn remove_white_king_side(&mut self) {
        self.castle_rights &= WHITE_KING_SIDE_REMOVED;
    }
    fn remove_white_queen_side(&mut self) {
        self.castle_rights &= WHITE_QUEEN_SIDE_REMOVED;
    }
    fn remove_black_king_side(&mut self) {
        self.castle_rights &= BLACK_KING_SIDE_REMOVED;
    }
    fn remove_black_queen_side(&mut self) {
        self.castle_rights &= BLACK_QUEEN_SIDE_REMOVED;
    }
    fn remove_castle_rights(&mut self, color: Color) {
        if color == Color::White {
            self.castle_rights &= WHITE_ALL_REMOVED;
        } else {
            self.castle_rights &= BLACK_ALL_REMOVED;
        }
    }
    pub fn make_move_new(&self, m: Move) -> Board {
        let mut new_board = *self;
        new_board.make_move(m);
        return new_board;
    }
    pub fn make_move(&mut self, m: Move) {
        let from = m.get_from();
        let to = m.get_to();
        let sp = m.get_sp();
        let piece = self.piece_on(from).unwrap();
        let target = self.piece_on(to);
        let me = self.turn;
        let them = !me;
        let offset = u8::max(from, to) - u8::min(from, to);

        if target.is_some() {
            self.remove_piece(target.unwrap(), to, them);
        }
        self.remove_piece(piece, from, me);
        if m.is_promotion() {
            self.put_piece(PROMOTION_PIECES[sp as usize], to, me);
        } else {
            self.put_piece(piece, to, me);
        }

        if m.is_ep() {
            self.remove_piece(Piece::Pawn, EP_TARGETS[to as usize], them);
        }

        self.hash ^= get_castle_zobrist(self.castle_rights);
        if piece == Piece::King {
            self.remove_castle_rights(me);
        }
        if from == WHITE_KING_SIDE_ROOK || to == WHITE_KING_SIDE_ROOK {
            self.remove_white_king_side();
        }
        if from == BLACK_KING_SIDE_ROOK || to == BLACK_KING_SIDE_ROOK {
            self.remove_black_king_side();
        }
        if from == WHITE_QUEEN_SIDE_ROOK || to == WHITE_QUEEN_SIDE_ROOK {
            self.remove_white_queen_side();
        }
        if from == BLACK_QUEEN_SIDE_ROOK || to == BLACK_QUEEN_SIDE_ROOK {
            self.remove_black_queen_side();
        }
        self.hash ^= get_castle_zobrist(self.castle_rights);

        if m.is_castle() {
            if sp == KING_SIDE_CASTLE {
                self.remove_piece(Piece::Rook, KING_SIDE_CASTLE_ROOKS[me as usize], me);
                self.put_piece(Piece::Rook, KING_SIDE_CASTLE_ROOK_DEST[me as usize], me);
            }
            if sp == QUEEN_SIDE_CASTLE {
                self.remove_piece(Piece::Rook, QUEEN_SIDE_CASTLE_ROOKS[me as usize], me);
                self.put_piece(Piece::Rook, QUEEN_SIDE_CASTLE_ROOK_DEST[me as usize], me);
            }
        }

        self.hash ^= get_ep_zobrist(!self.turn, self.en_passant);
        if piece == Piece::Pawn && offset == 16 {
            self.en_passant = Some((from + to) >> 1);
        } else {
            self.en_passant = None;
        }
        self.hash ^= get_ep_zobrist(self.turn, self.en_passant);

        self.turn = !self.turn;
        self.hash ^= get_turn_zobrist();

        if piece != Piece::Pawn && target.is_none() {
            self.halfmove_clock += 1;
        } else {
            self.halfmove_clock = 0;
        }
        self.update_pins_and_checks();
    }
    pub fn make_move_from_str(&mut self, str: &str) {
        let mut m = move_from_string(str);
        if self.piece_on(m.get_from()).unwrap() == Piece::King {
            match str {
                "e1g1" => m |= KING_SIDE_CASTLE_MOVES[0],
                "e8g8" => m |= KING_SIDE_CASTLE_MOVES[1],
                "e1c1" => m |= QUEEN_SIDE_CASTLE_MOVES[0],
                "e8c8" => m |= QUEEN_SIDE_CASTLE_MOVES[1],
                _ => {}
            }
        } else if self.piece_on(m.get_from()).unwrap() == Piece::Pawn {
            let offset = m.get_from().abs_diff(m.get_to());
            if offset == 7 || offset == 9 {
                if self.piece_on(m.get_to()).is_none() {
                    m |= EN_PASSANT << 12;
                }
            }
        }
        self.make_move(m);
    }
    #[allow(unused)]
    pub fn get_extended_move(&self, mv: Move) -> ExtendedMove {
        return ExtendedMove {
            from: mv.get_from(),
            to: mv.get_to(),
            piece: self.piece_on(mv.get_from()).unwrap_or(Piece::Pawn),
            special_falg: SpecialFalg::from_u16(mv.get_sp()),
        };
    }
}
	