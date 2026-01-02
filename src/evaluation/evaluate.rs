use crate::{
    core::{
        Board, Color::{self, Black, White}, Piece, bitboard::{
            BitBoard, BitBoardUtil, DARK_SQUARES, FILE_BITBOARDS, LIGHT_SQUARES, SECOND_RANK,
            SEVENTH_RANK,
        }, castle_rights::{CastleRightsUtil, Rights}, square::Square, tables::magics::{get_bishop_moves, get_knight_moves, get_rook_moves}
    },
    evaluation::{
        defs::{
            BISHOP_MOBILITY_SCORE, BISHOP_PAIR_VALUE, ENDGAME_MATERIAL_START,
            KNIGHT_MOBILITY_SCORE, MULTIPLIER, OPEN_UNHEALTHY_PAWN_PENALTY, PASSED_PAWN_VALUES,
            PAWN_STORM_PENALTY, PIECE_VALUES, ROOK_MOBILITY_SCORE_ENDGAME,
            ROOK_MOBILITY_SCORE_MIDDLEGAME, UNHEALTHY_PAWN_PENALTY,
        },
        tables::{
            KING_SIDE_CASTLE_FILESET, QUEEN_SIDE_CASTLE_FILESET, calc_king_pst, get_adjacent_files, get_adjacent_fileset, get_distance_from_center, get_fileset_bb, get_front_span, get_king_attacks, get_orthogonal_distance, get_pawn_endgame_value, get_pst_value
        },
    },
    transposition_table::{PawnEntry, TranspositionTable},
};

fn count_material(board: &Board, color_combined: u64) -> i16 {
    let queens = (board.pieces[4] & color_combined).count_ones() as i16;
    let rooks = (board.pieces[3] & color_combined).count_ones() as i16;
    let bishops = (board.pieces[2] & color_combined).count_ones() as i16;
    let knights = (board.pieces[1] & color_combined).count_ones() as i16;

    return queens * PIECE_VALUES[4]
        + rooks * PIECE_VALUES[3]
        + bishops * PIECE_VALUES[2]
        + knights * PIECE_VALUES[1];
}

fn get_endgame_weight(m: i16) -> f32 {
    let material = m as f32;
    if material < ENDGAME_MATERIAL_START {
        return 1.0 - (material * MULTIPLIER);
    } else {
        return 0.0;
    };
}
fn get_value<T>(m: T, e: T, endgame: f32) -> T {
    if endgame == 0.0 {
        return m;
    }
    return e;
}

pub fn evaluate(board: &Board, tt: &TranspositionTable) -> i16 {
    let white_combined = board.color_combined[0];
    let black_combined = board.color_combined[1];

    let wk = board.king_square(White);
    let bk = board.king_square(Black);
    let wq = board.get_piece_bitboard(Piece::Queen, White);
    let bq = board.get_piece_bitboard(Piece::Queen, Black);
    let wr = board.get_piece_bitboard(Piece::Rook, White);
    let br = board.get_piece_bitboard(Piece::Rook, Black);
    let wp = board.get_piece_bitboard(Piece::Pawn, White);
    let bp = board.get_piece_bitboard(Piece::Pawn, Black);

    let white_material_without_pawns = count_material(board, white_combined);
    let black_material_without_pawns = count_material(board, black_combined);

    let white_endgame = get_endgame_weight(white_material_without_pawns);
    let black_endgame = get_endgame_weight(black_material_without_pawns);

    let white_middlegame = 1.0 - white_endgame;
    let black_middlegame = 1.0 - black_endgame;

    let material_eval = white_material_without_pawns + PIECE_VALUES[0] * wp.count_ones() as i16
        - black_material_without_pawns
        - PIECE_VALUES[0] * bp.count_ones() as i16;

    let piece_scores = board.pst_value
        + calc_king_pst(0, wk, black_endgame, black_middlegame)
        + calc_king_pst(1, bk, white_endgame, white_middlegame);

    let mop_eval: i16 = mop_up_eval(
        wk,
        bk,
        white_material_without_pawns,
        black_material_without_pawns,
        black_endgame,
    ) - mop_up_eval(
        bk,
        wk,
        black_material_without_pawns,
        white_material_without_pawns,
        white_endgame,
    );

    let (pawn_eval, wp_fileset, bp_fileset) = evaluate_pawns(
        tt,
        board.pawn_hash,
        (white_endgame, black_endgame),
        (white_middlegame, black_middlegame),
        wp,
        bp,
    );

    let closed = wp_fileset & bp_fileset;
    let open = (!wp_fileset) & (!bp_fileset);
    let semi_open_white = bp_fileset & (!wp_fileset);
    let semi_open_black = wp_fileset & (bp_fileset);

    let rooks_eval = evaluate_rooks(
        wr,
        br,
        get_fileset_bb(open),
        get_fileset_bb(semi_open_white),
        get_fileset_bb(semi_open_black),
        get_fileset_bb(closed),
        wk as usize,
        bk as usize,
        (white_endgame, black_endgame),
    );

    let bishop_eval = evaluate_bishop_pair(board.get_piece_bitboard(Piece::Bishop, White))
        - evaluate_bishop_pair(board.get_piece_bitboard(Piece::Bishop, Black));

    let queens_eval = evaluate_queens(board.get_piece_bitboard(Piece::Queen, White), bk)
        - evaluate_queens(board.get_piece_bitboard(Piece::Queen, Black), wk);

    let seventh_rank_value = if (bp & SEVENTH_RANK) != 0 || bk > 55 {
        seventh_rank_bounus(wq & SEVENTH_RANK, wr & SEVENTH_RANK, black_endgame)
    } else {
        0
    } - if (wp & SECOND_RANK) != 0 || wk < 8 {
        seventh_rank_bounus(bq & SECOND_RANK, br & SECOND_RANK, white_endgame)
    } else {
        0
    };

    let (white_mobility_score, white_attack_count, white_attack_value) =
        evaluate_mobility(board, get_king_attacks(bk), White, black_endgame);
    let (black_mobility_score, black_attack_count, black_attack_value) =
        evaluate_mobility(board, get_king_attacks(wk), Black, white_endgame);

    let king_eval = if black_endgame != 0.0 && bq != 0 {
        evaluate_king_safety(
            wp,
            bp,
            wk as usize,
            0,
            board,
            black_attack_count,
            black_attack_value,
        )
    } else {
        0
    } - if white_endgame != 0.0 && wq != 0 {
        evaluate_king_safety(
            bp,
            wp,
            bk as usize,
            1,
            board,
            white_attack_count,
            white_attack_value,
        )
    } else {
        0
    };

    let tempo_bounus = if board.turn == White {
        get_value(20, 10, black_endgame)
    } else {
        get_value(-20, -10, white_endgame)
    };
    let eval = material_eval + white_mobility_score - black_mobility_score
        + mop_eval
        + piece_scores
        + pawn_eval
        + bishop_eval
        + rooks_eval
        + queens_eval
        + seventh_rank_value
        + tempo_bounus
        + king_eval;
    if board.turn == White {
        return eval;
    }
    return -eval;
}

fn mop_up_eval(
    my_king: Square,
    their_king: Square,
    my_material: i16,
    their_material: i16,
    endgame: f32,
) -> i16 {
    let mut score: i16 = 0;
    if my_material as f32 > (their_material as f32 + 200.0) && endgame > 0.0 {
        score += get_distance_from_center(their_king) as i16 * 10;
        score += (14 - get_orthogonal_distance(my_king, their_king) as i16) * 4
    }
    return score;
}

fn evaluate_pawns(
    tt: &TranspositionTable,
    hash: u64,
    endgame: (f32, f32),
    middle_game: (f32, f32),
    wp: u64,
    bp: u64,
) -> (i16, u8, u8) {
    let entry = tt.lookup_pawn_structure(hash);
    if entry.is_some() {
        let pawn_data = entry.unwrap();
        let score = pawn_data.eval
            + (pawn_data.w_pst.0 as f32 * middle_game.1
                + pawn_data.w_pst.1 as f32 * endgame.1
                + pawn_data.b_pst.0 as f32 * middle_game.0
                + pawn_data.b_pst.1 as f32 * endgame.0) as i16;
        return (score, pawn_data.w_filesets, pawn_data.b_filesets);
    } else {
        let w_data = get_pawn_data(wp, bp, White);
        let b_data = get_pawn_data(bp, wp, Black);
        let score = w_data.0 - b_data.0
            + (w_data.1 as f32 * middle_game.1
                + w_data.2 as f32 * endgame.1
                + b_data.1 as f32 * middle_game.0
                + b_data.2 as f32 * endgame.0) as i16;
        let e = PawnEntry{
            key: (hash >> 48) as u16,
            w_filesets: w_data.1,
            b_filesets: b_data.1,
            b_pst: (b_data.2, b_data.3),
            w_pst: (w_data.2, w_data.3),
            eval: score,
        };
        tt.store_pawn_structure(
            hash,
            e
        );
        return (score, w_data.1, b_data.1);
    };
}
fn get_pawn_data(pawns: BitBoard, enemy_pawns: BitBoard, color: Color) -> (i16, u8, i16, i16) {
    let mut score = 0;
    let mut p = pawns;
    let mut fileset: u8 = 0;
    let mut middle_game = 0;
    let mut endgame = 0;
    while p != 0 {
        let i = p.pop_lsb();
        let file = (i & 7) as usize;
        endgame += get_pawn_endgame_value(color, i);
        middle_game += get_pst_value(color, Piece::Pawn, i);
        let front_span = get_front_span(color, i) & enemy_pawns;
        let is_open = front_span & FILE_BITBOARDS[file] == 0;
        if ((fileset >> file) & 1) == 1 {
            //doubled pawn
            score -= if is_open {
                OPEN_UNHEALTHY_PAWN_PENALTY
            } else {
                UNHEALTHY_PAWN_PENALTY
            };
        } else {
            fileset |= 1 << file;
        }
        if front_span == 0 {
            //passer
            let rank = (i >> 3) as usize;
            score += PASSED_PAWN_VALUES[if color == White { 7 - rank } else { rank }]
        }
        if (get_adjacent_files(file) & pawns) == 0 {
            //isolated pawn
            score -= if is_open {
                OPEN_UNHEALTHY_PAWN_PENALTY
            } else {
                UNHEALTHY_PAWN_PENALTY
            };
        }
    }
    return (score, fileset, middle_game, endgame);
}
fn evaluate_rooks(
    wr: BitBoard,
    br: BitBoard,
    open: BitBoard,
    semi_open_white: BitBoard,
    semi_open_black: BitBoard,
    closed: BitBoard,
    wk: usize,
    bk: usize,
    endgame: (f32, f32),
) -> i16 {
    let mut score = 0;
    let w_adjacent = get_adjacent_files(wk & 7) & br;
    let b_adjacent = get_adjacent_files(bk & 7) & wr;
    let w_file = FILE_BITBOARDS[wk & 7] & br;
    let b_file = FILE_BITBOARDS[bk & 7] & wr;
    //closed files: -10
    score -= (closed & wr).count_ones() as i16 * 10 - (closed & br).count_ones() as i16 * 10;

    //open file
    score += (open & wr).count_ones() as i16 * 10
        + (open & b_adjacent).count_ones() as i16 * get_value(20, 10, endgame.1)
        + (open & b_file).count_ones() as i16 * get_value(30, 10, endgame.1);

    score -= (open & br).count_ones() as i16 * 10
        + (open & w_adjacent).count_ones() as i16 * get_value(20, 10, endgame.0)
        + (open & w_file).count_ones() as i16 * get_value(20, 10, endgame.0);

    if endgame.1 == 0.0 {
        score += (semi_open_white & b_adjacent).count_ones() as i16 * 10
            + (semi_open_white & b_file).count_ones() as i16 * 20;
    }
    if endgame.0 == 0.0 {
        score -= (semi_open_black & w_adjacent).count_ones() as i16 * 10
            + (semi_open_black & w_file).count_ones() as i16 * 20;
    }
    return score;
}
fn evaluate_bishop_pair(bishops: u64) -> i16 {
    if ((bishops & LIGHT_SQUARES) != 0) && ((bishops & DARK_SQUARES) != 0) {
        return BISHOP_PAIR_VALUE;
    }
    return 0;
}
fn evaluate_queens(mut queens: BitBoard, their_king: Square) -> i16 {
    let mut score = 0;
    while queens != 0 {
        score += 10 - get_orthogonal_distance(queens.to_sq(), their_king) as i16;
        queens &= queens - 1;
    }
    return score;
}
fn seventh_rank_bounus(queens: BitBoard, rooks: BitBoard, endgame: f32) -> i16 {
    return (rooks.count_ones() * get_value(10, 30, endgame)
        + queens.count_ones() * get_value(10, 20, endgame)) as i16;
}
fn evaluate_mobility(board: &Board, targets: u64, color: Color, endgame: f32) -> (i16, u32, f32) {
    let mut attacking_piece_count = 0;
    let mut attacking_piece_values = 0.0;
    let blockers = board.combined;

    let mut score = 0;

    let mut knights = board.get_piece_bitboard(Piece::Knight, color);
    let mut knight_moves = 0;
    while knights != 0 {
        let moves = get_knight_moves(knights.to_sq());
        knight_moves += (moves & !blockers).count_ones() as i16;
        if moves & targets != 0 {
            attacking_piece_count += 1;
            attacking_piece_values += 1.0;
        }
        knights &= knights - 1;
    }
    if knight_moves != 0 {
        score += (knight_moves - 4) * KNIGHT_MOBILITY_SCORE;
    }

    let mut bishops = board.get_piece_bitboard(Piece::Bishop, color);
    let mut bishop_moves = 0;
    while bishops != 0 {
        let moves = get_bishop_moves(bishops.to_sq(), blockers);
        bishop_moves += (moves & !blockers).count_ones() as i16;
        if moves & targets != 0 {
            attacking_piece_count += 1;
            attacking_piece_values += 1.0;
        }
        bishops &= bishops - 1;
    }
    if bishop_moves != 0 {
        score += (bishop_moves - 6) * BISHOP_MOBILITY_SCORE;
    }
    let mut rooks = board.get_piece_bitboard(Piece::Rook, color);
    let mut rook_moves = 0;
    while rooks != 0 {
        let moves = get_rook_moves(rooks.to_sq(), blockers);
        rook_moves += (moves & !blockers).count_ones() as i16;
        if moves & targets != 0 {
            attacking_piece_count += 1;
            attacking_piece_values += 2.0;
        }
        rooks &= rooks - 1;
    }
    if rook_moves != 0 {
        score += (rook_moves - 7)
            * get_value(
                ROOK_MOBILITY_SCORE_MIDDLEGAME,
                ROOK_MOBILITY_SCORE_ENDGAME,
                endgame,
            );
    }

    return (score, attacking_piece_count, attacking_piece_values);
}
fn evaluate_king_safety(
    my_pawns: BitBoard,
    their_pawns: BitBoard,
    king: usize,
    color: usize,
    board: &Board,
    attacking_pieces_count: u32,
    attacking_pieces_value: f32,
) -> i16 {
    let castling_rights = board
        .castle_rights
        .color(if color == 0 { White } else { Black });
    let mut storm_value = evaluate_pawn_storm(their_pawns, get_adjacent_fileset(king & 7), color);
    if castling_rights != Rights::NoRights {
        let value = if castling_rights == Rights::KingSide {
            evaluate_pawn_storm(their_pawns, KING_SIDE_CASTLE_FILESET, color)
        } else if castling_rights == Rights::QueenSide {
            evaluate_pawn_storm(their_pawns, QUEEN_SIDE_CASTLE_FILESET, color)
        } else {
            std::cmp::max(
                evaluate_pawn_storm(their_pawns, KING_SIDE_CASTLE_FILESET, color),
                evaluate_pawn_storm(their_pawns, QUEEN_SIDE_CASTLE_FILESET, color),
            )
        };
        storm_value = (storm_value + value) / 2;
    }
    return evaluate_pawn_shield(my_pawns, king, color) + storm_value
        - evaluate_piece_attacks(
            board,
            get_king_attacks(king as Square),
            attacking_pieces_count,
            attacking_pieces_value,
            if color == 0 { Black } else { White },
        );
}
fn evaluate_pawn_storm(their_pawns: u64, mut fileset: u8, color: usize) -> i16 {
    let mut score = 0;
    while fileset != 0 {
        let file = fileset.trailing_zeros() as usize;
        let bb = FILE_BITBOARDS[file] & their_pawns;
        fileset &= fileset - 1;
        if bb != 0 {
            let pawn = if color == 0 {
                bb.trailing_zeros()
            } else {
                63 - bb.leading_zeros()
            };
            score += PAWN_STORM_PENALTY[if color == 0 {
                (pawn >> 3) as usize
            } else {
                (7 - (pawn >> 3)) as usize
            }];
        }
    }
    return score;
}
fn evaluate_pawn_shield(pawns: u64, king: usize, color: usize) -> i16 {
    let mut score = 0;
    let mut fileset = get_adjacent_fileset(king & 7);
    let king_file = king >> 3;
    while fileset != 0 {
        let file = fileset.trailing_zeros() as usize;
        let file_bb = FILE_BITBOARDS[file] & pawns;
        fileset &= fileset - 1;
        let penalty = if file_bb == 0 {
            36
        } else {
            let pawn = if color == 0 {
                file_bb.trailing_zeros()
            } else {
                63 - file_bb.leading_zeros()
            };
            let distance_to_8 = if color == 0 {
                7 - (pawn >> 3)
            } else {
                pawn >> 3
            } as i16;
            36 - (distance_to_8 * distance_to_8)
        };
        if file == king_file {
            score -= penalty << 1;
        } else {
            score -= penalty;
        }
    }
    return score;
}
fn evaluate_piece_attacks(
    board: &Board,
    targets: u64,
    mut count: u32,
    mut values: f32,
    color: Color,
) -> i16 {
    let blockers = board.combined;
    let mut queens = board.get_piece_bitboard(Piece::Queen, color);
    while queens != 0 {
        let q = queens.pop_lsb();
        if (get_rook_moves(q, blockers) & targets != 0)
            || (get_bishop_moves(q, blockers) & targets != 0)
        {
            count += 1;
            values += 4.0;
        }
    }
    return (20.0 * values * get_piece_attack_weight(count)) as i16;
}
fn get_piece_attack_weight(num: u32) -> f32 {
    match num {
        0 => 0.0,
        1 => 0.0,
        2 => 0.5,
        3 => 0.75,
        4 => 0.88,
        5 => 0.94,
        6 => 0.97,
        7 => 0.99,
        _ => 1.0,
    }
}
