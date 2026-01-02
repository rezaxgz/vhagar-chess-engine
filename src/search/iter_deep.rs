use std::sync::Arc;

use crate::{core::{Board, r#move::Move}, search::{defs::SearchInfo, search::search_root_parallel}, transposition_table::TranspositionTable, uci::Uci};

pub fn start_iterative_deepening_search(
    board: &Board,
    tt: Arc<TranspositionTable>,
    search_info: &mut SearchInfo,
    threads: usize,
) -> Move {
    let mut best_move = None;

    for depth in 1..=search_info.max_depth {
        let mut result = search_root_parallel(
            board,
            depth,
            Arc::clone(&tt),
            threads,
        );

        best_move = Some(result.best_move);

        tt.calculate_pv(board.clone(), &mut result.pv);
        Uci::send_info(&result);
    }

    best_move.unwrap()
}
