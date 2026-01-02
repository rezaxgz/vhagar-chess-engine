use crate::{core::{Board, r#move::MoveUtil, movegen::generate_all_moves}, search::{defs::{MATE_SCORE, Score, ThreadData}, moves::{MoveType, sort_all_moves}, quiescence::quiescence}, transposition_table::{Flag, TTEntry, TranspositionTable}};

pub fn alpha_beta(
    board: &Board,
    mut alpha: Score,
    beta: Score,
    tt: &TranspositionTable,
    thread_data: &mut ThreadData
) -> Score {
    if thread_data.depth <= 0 {
        return quiescence(board, alpha, beta, tt, thread_data);
    }

    thread_data.nodes += 1;
    
    let key = board.hash;
    
    let mut tt_move = 0;
    if let Some(e) = tt.lookup_position(key) {
        thread_data.tt_hits += 1;
        if e.depth >= thread_data.depth {
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
            return MATE_SCORE + thread_data.ply as i16; // checkmate
        }
    }

    let mut move_types = vec![MoveType::BadCapture; movelist.len()];

    sort_all_moves(board, thread_data, tt_move, thread_data.ply, &mut movelist, &mut move_types);

    let alpha_orig = alpha;
    let mut best = i16::MIN + 10;
    let mut best_move = 0;

    thread_data.ply += 1;
    thread_data.depth -= 1;
    for i in 0..movelist.len() {
        let mv = movelist[i];
        let move_type = move_types[i];

        let new_board = board.make_move_new(mv);

        let score = -alpha_beta(&new_board, -beta, -alpha, tt, thread_data);

        if score > best {
            best = score;
            best_move = mv;
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            thread_data.beta_cutoffs += 1;

            if move_type == MoveType::QuietMove {
                thread_data.store_killer_move(thread_data.depth, thread_data.ply, mv, board.piece_on(mv.get_from()).unwrap(), board.turn);
            }
            break;
        }
        if move_type == MoveType::QuietMove {
            thread_data.store_bad_quiet(thread_data.depth, mv, board.piece_on(mv.get_from()).unwrap(), board.turn);
        }
    }
    thread_data.depth += 1;
    thread_data.ply -= 1;

    let flag = if best <= alpha_orig {
        Flag::UPPER
    } else if best >= beta {
        Flag::LOWER
    } else {
        Flag::EXACT
    };

    let entry = TTEntry {
        key: (key >> 48) as u16,
        depth: thread_data.depth,
        flag,
        eval: best,
        best_move,
        age: 0,
    };
    tt.store_position(key, entry);

    best
}
