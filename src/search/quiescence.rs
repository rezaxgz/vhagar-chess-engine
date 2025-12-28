use crate::{core::{Board, movegen::generate_quiescence_moves}, evaluation::evaluate::evaluate, transposition_table::TranspositionTable};

pub fn quiescence(
    board: &Board,
    alpha: i16,
    beta: i16,
    tt: &TranspositionTable,
) -> i16 {
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
    for mv in moves {
        let score = -quiescence(&board.make_move_new(mv), -beta, -alpha, tt);
        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }
    return alpha;
}
