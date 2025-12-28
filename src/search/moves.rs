use std::i32;

use crate::{
    core::{
        Board, Color, Piece, bitboard::BitBoardUtil, r#move::{Move, MoveUtil}
    },
    search::{defs::{Depth, ThreadData}, tables::get_sort_tabel_value},
};
type Score = i32;
#[derive(PartialEq, PartialOrd, Eq, Ord, Clone, Copy, Debug)]
pub enum MoveType {
    BadCapture, //captures with a piece of higher value that can be recaptured by a pawn
    QuietMove,
    KillerMove,
    GoodCapture,
    Promotion,
    HashMove,
}

const NOT_FILE_A_BB: u64 = !72340172838076673;
const NOT_FILE_H_BB: u64 = !(72340172838076673 << 7);
const PROMOTION_VALUES: [Score; 6] = [0, 8, 8, 16, 32, 0];
const MVV_LVA: [[Score; 6]; 5] = [
    [15, 14, 13, 12, 11, 10], // victim P, attacker P, N, B, R, Q, K
    [25, 24, 23, 22, 21, 20], // victim N, attacker P, N, B, R, Q, K
    [35, 34, 33, 32, 31, 30], // victim B, attacker P, N, B, R, Q, K
    [45, 44, 43, 42, 41, 40], // victim R, attacker P, N, B, R, Q, K
    [55, 54, 53, 52, 51, 50], // victim Q, attacker P, N, B, R, Q, K
];

fn piece_value(piece: Piece) -> i8 {
    match piece {
        Piece::Pawn => 1,
        Piece::King => 0,
        Piece::Bishop => 3,
        Piece::Knight => 3,
        Piece::Rook => 5,
        Piece::Queen => 9,
    }
}
fn move_type(
    piece_at_start: Piece,
    piece_at_end: Option<Piece>,
    is_controled: bool,
    is_tt_move: bool,
    is_killer: bool,
    is_promo: bool,
) -> MoveType {
    if is_tt_move {
        return MoveType::HashMove;
    }
    if is_promo {
        return MoveType::Promotion;
    }
    if piece_at_end.is_some() {
        if is_controled && piece_value(piece_at_start) > piece_value(piece_at_end.unwrap()) {
            return MoveType::BadCapture;
        }
        return MoveType::GoodCapture;
    }
    if is_killer {
        return MoveType::KillerMove;
    }
    return MoveType::QuietMove;
}
fn set_move_value(
    m: Move,
    piece_at_start: Piece,
    piece_at_end: Option<Piece>,
    is_controled: bool,
    color: Color,
    is_tt_move: bool,
    is_killer: bool,
    history_value: i32,
    thread_data: &mut ThreadData,
) {
    let mt = move_type(
        piece_at_start,
        piece_at_end,
        is_controled,
        is_tt_move,
        is_killer,
        m.is_promotion(),
    );
    let mut value = 0;
    if piece_at_end.is_some() {
        //captures sorted with MVV_LVA
        value += MVV_LVA[piece_at_end.unwrap() as usize][piece_at_start as usize];
    } else {
        //quiets sorter with history heuristic
        if is_controled && piece_at_start != Piece::Pawn {
            value -= 1000000;
        }
        value += history_value;
    }

    if m.is_promotion() {
        value += PROMOTION_VALUES[m.get_sp() as usize];
    } else {
        value += (get_sort_tabel_value(piece_at_start, m.get_to(), color)
            - get_sort_tabel_value(piece_at_start, m.get_from(), color)) as i32;
    }
    thread_data.move_values[m as usize] = value;
    thread_data.move_types[m as usize] = mt;
    
}
pub fn sort_all_moves(
    board: &Board,
    thread_data: &mut ThreadData,
    tt_move: Move,
    ply: Depth,
    moves: &mut Vec<Move>,
    move_types: &mut Vec<MoveType>,
) {
    let killers = thread_data.get_killers(ply);
    let pawns = board.get_piece_bitboard(Piece::Pawn, !board.turn);
    let controled = if board.turn == Color::White {
        (pawns >> 9 & NOT_FILE_H_BB) | (pawns >> 7 & NOT_FILE_A_BB)
    } else {
        (pawns << 7 & NOT_FILE_H_BB) | (pawns << 9 & NOT_FILE_A_BB)
    };
    for i in 0..moves.len() {
        let mv = moves[i];
        let p = board.piece_on(mv.get_from()).unwrap();
        set_move_value(
            mv,
            p,
            board.piece_on(mv.get_to()),
            controled.has_sq(mv.get_to()),
            board.turn,
            mv == tt_move,
            killers.contains(&mv),
            thread_data.history[board.turn as usize][p as usize][mv.get_to() as usize],
            thread_data,
        );
    }
    moves.sort_by(|b, a| {
        if thread_data.move_types[*a as usize] != thread_data.move_types[*b as usize] {
            thread_data.move_types[*a as usize].cmp(&thread_data.move_types[*b as usize])
        } else {
            thread_data.move_values[*a as usize].cmp(&thread_data.move_values[*b as usize])
        }
    });
    for i in 0..moves.len() {
        move_types[i] = thread_data.move_types[moves[i] as usize];
    }
}
fn set_capture_value(mv: Move, piece: Piece, captured: Piece, is_controled: bool, thread_data: &mut ThreadData) {
    let mut value = MVV_LVA[captured as usize][piece as usize];
    if is_controled && piece != Piece::Pawn && piece_value(piece) > piece_value(captured) {
        value -= 50;
    } else if mv.is_promotion() {
        value += PROMOTION_VALUES[mv.get_sp() as usize];
    }
    thread_data.move_values[mv as usize] = value;
    
}
pub fn sort_captures(board: &Board, moves: &mut Vec<Move>,thread_data: &mut ThreadData) {
    let pawns = board.get_piece_bitboard(Piece::Pawn, !board.turn);
    let controled = if board.turn == Color::White {
        (pawns >> 9 & NOT_FILE_H_BB) | (pawns >> 7 & NOT_FILE_A_BB)
    } else {
        (pawns << 7 & NOT_FILE_H_BB) | (pawns << 9 & NOT_FILE_A_BB)
    };
    for i in 0..moves.len() {
        let mv = moves[i];
        set_capture_value(
            mv,
            board.piece_on(mv.get_from()).unwrap(),
            board.piece_on(mv.get_to()).unwrap_or(Piece::Pawn),
            controled.has_sq(mv.get_to()),
            thread_data
        );
    }
    moves.sort_by(|b, a| thread_data.move_values[*a as usize].cmp(&thread_data.move_values[*b as usize]));
}
