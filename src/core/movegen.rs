use crate::core::{
    bitboard::{BitBoard, BitBoardUtil, FULL_BITBOARD, RANK_BITBOARDS},
    castle_rights::CastleRightsUtil,
    r#move::{
        new_move, Move, EN_PASSANT, KING_SIDE_CASTLE_MOVES, KNIGHT_PROMOTION, QUEEN_PROMOTION,
        QUEEN_SIDE_CASTLE_MOVES,
    },
    square::{Square, SquareUtil},
    tables::magics::{
        get_between, get_bishop_moves, get_king_moves, get_knight_moves, get_line,
        get_pawn_attacks, get_pawn_moves, get_rook_moves, KINGSIDE_CASTLE_SQUARES,
        QUEENSIDE_CASTLE_SQUARES, QUEEN_SIDE_CASTLE_SQUARES_CONTROLLED,
    },
    Board, Color, Piece,
};

fn generate_king_moves(board: &Board, king: Square, movelist: &mut Vec<Move>) {
    let mut moves = get_king_moves(king) & !board.get_friendly_pieces();
    let mut checkers = board.checkers;
    while checkers != 0 {
        let sq = checkers.pop_lsb();
        if board.piece_on(sq).unwrap().is_sliding() {
            moves &= !get_line(king, sq) | board.checkers;
        }
    }
    while moves != 0 {
        let dest = moves.pop_lsb();
        if !board.is_controlled(dest, !board.turn) {
            movelist.push(new_move(king, dest, 0));
        }
    }
}

fn add_pawn_moves(from: Square, to: Square, movelist: &mut Vec<Move>) {
    if to > 55 || to < 8 {
        for i in KNIGHT_PROMOTION..=QUEEN_PROMOTION {
            movelist.push(new_move(from, to, i));
        }
        return;
    }
    movelist.push(new_move(from, to, 0));
}

fn generate_pawn_moves(board: &Board, movelist: &mut Vec<Move>, mut mask: BitBoard, king: Square) {
    let mut pawns = board.get_piece_bitboard(Piece::Pawn, board.turn);
    let mut enemy_pieces = board.get_enemy_pieces();
    let ep = if board.en_passant.is_some() {
        enemy_pieces |= 1 << board.en_passant.unwrap();
        board.en_passant.unwrap()
    } else {
        64
    };
    if board.en_passant.is_some() && (board.checkers & board.pieces[Piece::Pawn as usize] != 0) {
        mask |= 1 << ep;
    }
    while pawns != 0 {
        let pawn = pawns.pop_lsb();

        //generate captures
        let mut capture_targets = get_pawn_attacks(pawn, board.turn) & enemy_pieces & mask;
        if board.pinned.has_sq(pawn) {
            capture_targets &= get_line(pawn, king);
        }
        'l: while capture_targets != 0 {
            let target = capture_targets.pop_lsb();
            if target == ep {
                if king.get_rank() == pawn.get_rank() {
                    let mut rooks = board.get_line_attackers(!board.turn)
                        & RANK_BITBOARDS[king.get_rank() as usize];
                    while rooks != 0 {
                        if (get_between(rooks.pop_lsb(), king) & board.combined).count_ones() == 2 {
                            continue 'l;
                        }
                    }
                }
                movelist.push(new_move(pawn, target, EN_PASSANT));
                continue 'l;
            }
            add_pawn_moves(pawn, target, movelist);
        }

        //generate quiets
        let mut quiet_targets = get_pawn_moves(pawn, board.turn) & !board.combined & mask;
        if board.pinned.has_sq(pawn) {
            quiet_targets &= get_line(pawn, king);
        }
        if board.turn == Color::White && (board.combined.has_sq(pawn + 8))
            || board.turn == Color::Black && (board.combined.has_sq(pawn - 8))
        {
            continue;
        }
        while quiet_targets != 0 {
            add_pawn_moves(pawn, quiet_targets.pop_lsb(), movelist);
        }
    }
}

fn generate_diagonal_moves(board: &Board, king: Square, mask: BitBoard, movelist: &mut Vec<Move>) {
    let mut pieces = (board.pieces[Piece::Bishop as usize] | board.pieces[Piece::Queen as usize])
        & board.get_friendly_pieces();
    let available_squares = !board.get_friendly_pieces();
    while pieces != 0 {
        let piece = pieces.pop_lsb();
        let mut dest = get_bishop_moves(piece, board.combined) & mask & available_squares;
        if board.pinned.has_sq(piece) {
            dest &= get_line(piece, king)
        }
        while dest != 0 {
            movelist.push(new_move(piece, dest.pop_lsb(), 0));
        }
    }
}

fn generate_line_moves(board: &Board, king: Square, mask: BitBoard, movelist: &mut Vec<Move>) {
    let mut pieces = (board.pieces[Piece::Rook as usize] | board.pieces[Piece::Queen as usize])
        & board.get_friendly_pieces();
    let available_squares = !board.get_friendly_pieces();
    while pieces != 0 {
        let piece = pieces.pop_lsb();
        let mut dest = get_rook_moves(piece, board.combined) & mask & available_squares;
        if board.pinned.has_sq(piece) {
            dest &= get_line(piece, king)
        }
        while dest != 0 {
            movelist.push(new_move(piece, dest.pop_lsb(), 0));
        }
    }
}

fn generate_knight_moves(board: &Board, movelist: &mut Vec<Move>, mask: BitBoard) {
    let mut knights = board.get_piece_bitboard(Piece::Knight, board.turn) & !board.pinned;
    let available_squares = !board.get_friendly_pieces();
    while knights != 0 {
        let knight = knights.pop_lsb();
        let mut dest = get_knight_moves(knight) & mask & available_squares;
        while dest != 0 {
            movelist.push(new_move(knight, dest.pop_lsb(), 0));
        }
    }
}

fn generate_castle_moves(board: &Board, movelist: &mut Vec<Move>) {
    if board.checkers != 0 {
        return;
    }
    if board.castle_rights.has_king_side(board.turn)
        && ((KINGSIDE_CASTLE_SQUARES[board.turn as usize] & board.combined) == 0)
    {
        let mut is_legal = true;
        let mut squares = KINGSIDE_CASTLE_SQUARES[board.turn as usize];
        while squares != 0 {
            if board.is_controlled(squares.pop_lsb(), !board.turn) {
                is_legal = false;
                break;
            }
        }
        if is_legal {
            movelist.push(KING_SIDE_CASTLE_MOVES[board.turn as usize]);
        }
    }
    if board.castle_rights.has_queen_side(board.turn)
        && ((QUEENSIDE_CASTLE_SQUARES[board.turn as usize] & board.combined) == 0)
    {
        let mut is_legal = true;
        let mut squares = QUEEN_SIDE_CASTLE_SQUARES_CONTROLLED[board.turn as usize];
        while squares != 0 {
            if board.is_controlled(squares.pop_lsb(), !board.turn) {
                is_legal = false;
                break;
            }
        }
        if is_legal {
            movelist.push(QUEEN_SIDE_CASTLE_MOVES[board.turn as usize]);
        }
    }
}

pub fn generate_all_moves(board: &Board, movelist: &mut Vec<Move>) {
    let king = board.king_square(board.turn);

    let mask = if board.checkers == 0 {
        FULL_BITBOARD
    } else if board.checkers.count_ones() == 1 {
        get_between(
            board.checkers.to_sq(),
            board.get_piece_bitboard(Piece::King, board.turn).to_sq(),
        ) | board.checkers
    } else {
        generate_king_moves(board, king, movelist);
        return;
    };

    generate_castle_moves(board, movelist);
    generate_pawn_moves(board, movelist, mask, king);
    generate_diagonal_moves(board, king, mask, movelist);
    generate_line_moves(board, king, mask, movelist);
    generate_knight_moves(board, movelist, mask);
    generate_king_moves(board, king, movelist);
}
#[allow(unused)]
pub fn generate_quiescence_moves(board: &Board, movelist: &mut Vec<Move>) {
    let king = board.king_square(board.turn);

    let mask = if board.checkers == 0 {
        board.get_enemy_pieces()
    } else if board.checkers.count_ones() == 1 {
        get_between(
            board.checkers.to_sq(),
            board.get_piece_bitboard(Piece::King, board.turn).to_sq(),
        ) | board.checkers
    } else {
        generate_king_moves(board, king, movelist);
        return;
    };
    generate_pawn_moves(board, movelist, mask, king);
    generate_diagonal_moves(board, king, mask, movelist);
    generate_line_moves(board, king, mask, movelist);
    generate_knight_moves(board, movelist, mask);
}
