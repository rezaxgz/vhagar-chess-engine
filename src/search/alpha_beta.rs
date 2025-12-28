use crate::{core::{Board, r#move::MoveUtil, movegen::generate_all_moves}, search::{defs::{Depth, MATE_SCORE, Score, ThreadData}, moves::{MoveType, sort_all_moves}, quiescence::quiescence}, transposition_table::{Flag, TTEntry, TranspositionTable}};

pub fn alpha_beta(
    board: &Board,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    tt: &TranspositionTable,
    ply: Depth,
    thread_data: &mut ThreadData
) -> Score {
    if depth == 0 {
        return quiescence(board, alpha, beta, tt, thread_data);
    }

    let key = board.hash;
    
    let mut tt_move = 0;
    if let Some(e) = tt.lookup_position(key) {
        if e.depth >= depth {
            match e.flag {
                Flag::EXACT => return e.eval,
                Flag::LOWER if e.eval >= beta => return beta, 
                Flag::UPPER if e.eval <= alpha => return alpha,
                _ => {}
            }
        }
        tt_move = e.best_move;
    }

    let mut movelist = Vec::with_capacity(100);

    generate_all_moves(board, &mut movelist);
    
    if movelist.is_empty() {
        if board.checkers == 0 {
            return 0; // stalemate
        } else {
            return MATE_SCORE + ply as i16; // checkmate
        }
    }

    let mut move_types = vec![MoveType::BadCapture; movelist.len()];

    sort_all_moves(board, thread_data, tt_move, ply, &mut movelist, &mut move_types);


    let mut best = i16::MIN;
    let mut best_move = 0;

    for i in 0..movelist.len() {
        let mv = movelist[i];
        let move_type = move_types[i];

        let new_board = board.make_move_new(mv);

        let score = -alpha_beta(&new_board, depth - 1, -beta, -alpha, tt, ply + 1, thread_data);

        if score > best {
            best = score;
            best_move = mv;
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            if move_type == MoveType::QuietMove {
                thread_data.store_killer_move(depth, ply, mv, board.piece_on(mv.get_from()).unwrap(), board.turn);
            }
            break;
        }
        if move_type == MoveType::QuietMove {
            thread_data.store_bad_quiet(depth, mv, board.piece_on(mv.get_from()).unwrap(), board.turn);
        }
    }

    let flag = if best <= alpha {
        Flag::UPPER
    } else if best >= beta {
        Flag::LOWER
    } else {
        Flag::EXACT
    };

    let entry = TTEntry {
        key: (key >> 48) as u16,
        depth: depth as i8,
        flag,
        eval: best,
        best_move,
        age: 0,
    };
    tt.store_position(key, entry);

    best
}
