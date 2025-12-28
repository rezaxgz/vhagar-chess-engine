use std::sync::{Arc, Mutex};
use std::thread;

use crate::core::Board;
use crate::core::movegen::generate_all_moves;
use crate::search::alpha_beta::alpha_beta;
use crate::search::defs::{Depth, RootResult, Score, ThreadData};
use crate::transposition_table::TranspositionTable;

pub fn search_root_parallel(
    board: &Board,
    depth: Depth,
    tt: Arc<TranspositionTable>,
    threads: usize,
) -> RootResult {
    let mut moves = Vec::with_capacity(218);
    generate_all_moves(board, &mut moves);
    // Shared best result
    let best = Arc::new(Mutex::new(RootResult {
        mv: moves[0],
        score: Score::MIN,
        
    }));

    // Split root moves into chunks
    let chunk_size = (moves.len() + threads - 1) / threads;

    let mut handles = Vec::new();

    for chunk in moves.chunks(chunk_size) {
        let board = board.clone();
        let tt = Arc::clone(&tt);
        let best = Arc::clone(&best);
        let chunk = chunk.to_vec();

        handles.push(thread::spawn(move || {
            let mut thread_data = ThreadData::new();
            for mv in chunk {

                let mut p = board.clone();
                p.make_move(mv);

                let score = -alpha_beta(
                    &p,
                    depth - 1,
                    Score::MIN,
                    Score::MAX,
                    &tt,
                    1,
                    &mut thread_data
                );

                let mut best_guard = best.lock().unwrap();
                if score > best_guard.score {
                    best_guard.score = score;
                    best_guard.mv = mv;
                }
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    best.lock().unwrap().clone()
}
