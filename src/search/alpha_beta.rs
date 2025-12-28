use crate::{core::{Board, movegen::generate_all_moves}, search::{defs::{Depth, Score}, quiescence::quiescence}, transposition_table::{Flag, TTEntry, TranspositionTable}};

pub fn alpha_beta(
    board: &Board,
    depth: Depth,
    mut alpha: Score,
    beta: Score,
    tt: &TranspositionTable,
    ply: Depth,
) -> Score {
    if depth == 0 {
        return quiescence(board, alpha, beta, tt);
    }

    let key = board.hash;

    if let Some(e) = tt.lookup_position(key) {
        if e.depth >= depth {
            match e.flag {
                Flag::EXACT => return e.eval,
                Flag::LOWER if e.eval >= beta => return beta, 
                Flag::UPPER if e.eval <= alpha => return alpha,
                _ => {}
            }
        }
    }

    let mut movelist = Vec::with_capacity(100);

    generate_all_moves(board, &mut movelist);


    let mut best = i16::MIN;
    let mut best_move = 0;

    for mv in movelist {
        let new_board = board.make_move_new(mv);

        let score = -alpha_beta(&new_board, depth - 1, -beta, -alpha, tt, ply + 1);

        if score > best {
            best = score;
            best_move = mv;
        }
        alpha = alpha.max(score);
        if alpha >= beta {
            break;
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
