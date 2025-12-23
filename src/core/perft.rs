use std::time::Instant;

use crate::core::{
    board::Board,
    movegen::generate_all_moves,
    r#move::{Move, MoveUtil},
};

pub fn perft(board: &Board, depth: usize, moves: &mut Vec<Move>) -> usize {
    if depth == 0 {
        return 1;
    }
    let mut nodes = 0;
    let mut all_moves = Vec::with_capacity(40);
    generate_all_moves(board, &mut all_moves);
    if depth == 1 {
        return all_moves.len();
    }
    for m in all_moves {
        let new_board = board.make_move_new(m);
        moves.push(m);
        nodes += perft(&new_board, depth - 1, moves);
        moves.pop();
    }
    return nodes;
}
pub fn start_perft(board: &Board, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
    let start = Instant::now();
    let mut nodes = 0;
    let mut all_moves = Vec::with_capacity(40);
    generate_all_moves(board, &mut all_moves);
    for m in all_moves {
        let new_board = board.make_move_new(m);
        let n = perft(&new_board, depth - 1, &mut vec![]);
        println!("{}: {}", m.to_str(), n);
        nodes += n;
    }

    println!("{} nodes searched in {:?}", nodes, start.elapsed());
    let secs = start.elapsed().as_micros() as f64 / 1000000.0;
    println!("{} nodes per second", (nodes as f64 / secs).floor());
    return nodes;
}
