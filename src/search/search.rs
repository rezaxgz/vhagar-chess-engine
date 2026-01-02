use std::sync::{Arc, Mutex};
use std::thread;

use crate::core::Board;
use crate::core::movegen::generate_all_moves;
use crate::search::alpha_beta::alpha_beta;
use crate::search::defs::{Depth, SearchResult, Score, ThreadData};
use crate::search::moves::sort_root_moves;
use crate::transposition_table::{Flag, TTEntry, TranspositionTable};

pub fn search_root_parallel(
    board: &Board,
    depth: Depth,
    tt: Arc<TranspositionTable>,
    threads: usize,
) -> SearchResult {
    let mut moves = Vec::with_capacity(218);
    generate_all_moves(board, &mut moves);

    if moves.is_empty(){
        if board.checkers == 0{
            return SearchResult::stalemate();
        }
        return SearchResult::checkmate();
    }

    sort_root_moves(board, tt.lookup_position(board.hash).unwrap_or(TTEntry::default()).best_move, &mut moves);
    // Shared best result
    let best = Arc::new(Mutex::new(SearchResult::inital()));
    // Split root moves into chunks
    let chunk_size = (moves.len() + threads - 1) / threads;

    let mut handles = Vec::new();

    for chunk in moves.chunks(chunk_size) {
        let board = board.clone();
        let tt = Arc::clone(&tt);
        let best = Arc::clone(&best);
        best.lock().unwrap().depth = depth;
        
        let chunk = chunk.to_vec();

        handles.push(thread::spawn(move || {
            let mut thread_data = ThreadData::new();
            thread_data.depth = depth;
            thread_data.ply = 1;

            let mut local_best = SearchResult::inital();

            for mv in chunk {
                let mut p = board.clone();
                p.make_move(mv);

                let score = -alpha_beta(
                    &p,
                    local_best.eval,
                    Score::MAX,
                    &tt,
                    &mut thread_data,
                );
                if score > local_best.eval {
                    local_best.eval = score;
                    local_best.best_move = mv;
                }
            }

            // Merge result
            let mut global = best.lock().unwrap();
            global.update(&local_best);
            global.update_stats(&thread_data);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
    let mut final_result = best.lock().unwrap();
    let entry = TTEntry{
        age: 0,
        key: (board.hash >> 48) as u16,
        best_move: final_result.best_move,
        depth: depth,
        eval: final_result.eval,
        flag: Flag::EXACT,
    };
    tt.store_position(board.hash, entry);
    final_result.update_timer();
    final_result.clone()
}
