use crate::{core::{Board, movegen::generate_quiescence_moves}, evaluation::evaluate::evaluate, search::{defs::ThreadData, moves::sort_captures}, transposition_table::TranspositionTable};

pub fn quiescence(
    board: &Board,
    alpha: i16,
    beta: i16,
    tt: &TranspositionTable,
    thread_data: &mut ThreadData
) -> i16 {
    thread_data.q_nodes += 1;
    
    let mut alpha = alpha;
    let stand_pat = evaluate(board, tt);
    if stand_pat >= beta {
        return beta;
    }
    if alpha < stand_pat {
        alpha = stand_pat;
    }
    let mut moves = Vec::with_capacity(20);
    generate_quiescence_moves(board, &mut moves);
    sort_captures(board, &mut moves, thread_data);
    for mv in moves {
        let score = -quiescence(&board.make_move_new(mv), -beta, -alpha, tt, thread_data);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    return alpha;
}
