use std::sync::Arc;

use crate::{core::{Board, r#move::{Move, MoveUtil}}, search::{defs::{Score, SearchInfo}, search::search_root_parallel}, transposition_table::TranspositionTable};

pub fn start_iterative_deepening_search(
    board: &Board,
    tt: Arc<TranspositionTable>,
    search_info: &mut SearchInfo,
    threads: usize,
) -> Move {
    let mut best_move = None;
    let mut best_score = Score::MIN;

    for depth in 1..=search_info.max_depth {
        let result = search_root_parallel(
            board,
            depth,
            Arc::clone(&tt),
            threads,
        );

        if result.score > best_score {
            best_score = result.score;
            best_move = Some(result.mv);
        }

        println!(
            "info depth {} score cp {} pv {}",
            depth, best_score, best_move.unwrap().to_str()
        );

        // (optional) aspiration windows would go here
    }

    best_move.unwrap()
}
