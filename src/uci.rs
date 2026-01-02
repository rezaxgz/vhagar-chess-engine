use std::sync::Arc;

use crate::{core::{Board, Color, r#move::MoveUtil, perft::start_perft, perft_test:: test_perft}, search::{defs::{SearchInfo, SearchMode, SearchResult}, iter_deep::start_iterative_deepening_search}, transposition_table::TranspositionTable, uci_options::UciOptions};
const ENGINENAME: &str = "Vhagar";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Reza Ghazavi";

const DEAFAULT_TT_SIZE_MB: usize = 64;

fn allocate_time(my_time: u128, my_inc: u128, moves_to_go: usize) -> (u128, u128) {
    let soft_bound = (my_time / u128::min(20, moves_to_go as u128)) + (my_inc / 2);
    let hard_bound = soft_bound + soft_bound / 5;
    return (soft_bound.min(my_time), hard_bound.min(my_time));
}
pub struct Uci {
    board: Board,
    position_cmd: String,
    tt: Arc<TranspositionTable>,
    options: UciOptions,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            position_cmd: String::from("position startpos moves"),
            tt: Arc::new(TranspositionTable::new(DEAFAULT_TT_SIZE_MB)),
            options: UciOptions::new()
        }
    }

    pub fn receive_cmd(&mut self, input: &str) {
        // Trim CR/LF so only the usable characters remain.
        let i = input.trim_end().to_string();

        // Convert to &str for matching the command.
        match i {
            // UCI commands
            cmd if cmd == "uci" => self.uciok(),
            cmd if cmd == "ucinewgame" => self.new_game(),
            cmd if cmd == "isready" => self.readyok(),
            cmd if cmd == "quit" || cmd == "exit" => self.quit(),
            cmd if cmd.starts_with("position") => self.parse_position(&cmd),
            cmd if cmd == "board" => self.print_board(),
            cmd if cmd.starts_with("perft") => self.parse_perft(&cmd),
            cmd if cmd.starts_with("go perft") => self.parse_perft(&cmd[3..]),
            cmd if cmd.starts_with("go") => self.parse_go(&cmd),
            cmd if cmd.starts_with("setoption") => self.parse_setoption(&cmd),
            // cmd if cmd == "bench" || cmd == "benchmark" => benchmark(),

            // Everything else is ignored.
            _ => {}
        }
    }

    fn parse_perft(&self, cmd: &str){
        println!("{}", cmd);
        let mut depth = 0;
        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        if parts.len() > 1 {
            if parts[1] == "test" {
                test_perft();
                return;
            }
            depth = parts[1].parse::<usize>().unwrap_or(1);
        }
        start_perft(&self.board, depth);
    }

    fn parse_position(&mut self, cmd: &str) {
        self.position_cmd = String::from(cmd);
        enum Tokens {
            Nothing,
            Fen,
            Moves,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut fen = String::from("");
        let mut moves: Vec<String> = Vec::new();
        let mut skip_fen = false;
        let mut token = Tokens::Nothing;

        for p in parts {
            match p {
                t if t == "position" => (), // Skip. We know we're parsing "position".
                t if t == "startpos" => {
                    skip_fen = true;
                    self.board = Board::default();
                } // "fen" is now invalidated.
                t if t == "fen" && !skip_fen => token = Tokens::Fen,
                t if t == "moves" => token = Tokens::Moves,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Fen => {
                        fen.push_str(&p[..]);
                        fen.push(' ');
                    }
                    Tokens::Moves => moves.push(p),
                },
            }
        }
        // No FEN part in the command. Use the start position.
        if fen.is_empty() {
            fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
        }
        self.board = Board::from_fen(&fen);
        for m in moves {
            self.board.make_move_from_str(&m);
        }
    }

    fn parse_go(&mut self, cmd: &str) {
        enum Tokens {
            Nothing,
            Depth,
            MoveTime,
            WTime,
            BTime,
            WInc,
            BInc,
            MovesToGo,
        }

        let parts: Vec<String> = cmd.split_whitespace().map(|s| s.to_string()).collect();
        let mut token = Tokens::Nothing;
        let mut info = SearchInfo::default();
        for p in parts {
            match p {
                t if t == "go" => {}
                t if t == "infinite" => info.search_mode = SearchMode::Infinite,
                t if t == "depth" => token = Tokens::Depth,
                t if t == "movetime" => token = Tokens::MoveTime,
                t if t == "wtime" => token = Tokens::WTime,
                t if t == "btime" => token = Tokens::BTime,
                t if t == "winc" => token = Tokens::WInc,
                t if t == "binc" => token = Tokens::BInc,
                t if t == "movestogo" => token = Tokens::MovesToGo,
                _ => match token {
                    Tokens::Nothing => (),
                    Tokens::Depth => {
                        info.max_depth = p.parse::<i8>().unwrap_or(1);
                        info.search_mode = SearchMode::Infinite;
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::MoveTime => {
                        info.max_move_time = p.parse::<u128>().unwrap_or(1000) - 5;
                        info.allocated_time = p.parse::<u128>().unwrap_or(1000) - 5;
                        info.search_mode = SearchMode::MoveTime;
                        break; // break for-loop: nothing more to do.
                    }
                    Tokens::WTime => info.game_time.wtime = p.parse::<u128>().unwrap_or(0),
                    Tokens::BTime => info.game_time.btime = p.parse::<u128>().unwrap_or(0),
                    Tokens::WInc => info.game_time.winc = p.parse::<u128>().unwrap_or(0),
                    Tokens::BInc => info.game_time.binc = p.parse::<u128>().unwrap_or(0),
                    Tokens::MovesToGo => {
                        info.game_time.moves_to_go = if let Ok(x) = p.parse::<usize>() {
                            Some(x)
                        } else {
                            None
                        }
                    }
                }, // end match token
            } // end match p
        } // end for
        if self.board.turn == Color::White && info.game_time.wtime != 0 {
            (info.allocated_time, info.max_move_time) = allocate_time(
                info.game_time.wtime,
                info.game_time.winc,
                info.game_time.moves_to_go.unwrap_or(20),
            );
        } else if self.board.turn == Color::Black && info.game_time.btime != 0 {
            (info.allocated_time, info.max_move_time) = allocate_time(
                info.game_time.btime,
                info.game_time.binc,
                info.game_time.moves_to_go.unwrap_or(20),
            );
        }

        let best_move = start_iterative_deepening_search(
            &self.board,
            Arc::clone(&self.tt),
            &mut info,
            self.options.thread_cout(),
        );
        println!("bestmove {}", best_move.to_str());
    } // end parse_go()

    fn parse_setoption(&mut self, cmd: &str){
        let args = cmd.split(" ").map(|a| String::from(a)).collect::<Vec<String>>();
        let mut i = 0;
        let mut name = String::new();
        let mut value = String::new();
        while i < args.len(){
            if i > 1 && args[i - 1] == "name"{
                name = args[i].clone();
            }
            if i > 1 && args[i - 1] == "value"{
                value = args[i].clone();
            }
            i += 1;
        }
        self.options.set(name, value);
    }

    fn id(&self) {
        println!("id name {} {}", ENGINENAME, VERSION);
        println!("id author {}", AUTHOR);
    }
    fn uciok(&self) {
        self.id();
        self.options.print();
        println!("uciok");
    }
    fn readyok(&self) {
        println!("readyok");
    }
    fn quit(&self) {
        std::process::exit(0);
    }
    fn print_board(&self) {
        self.board.print();
    }
    fn new_game(&mut self) {
        *self = Uci::new();
    }
}
impl Uci {
    pub fn connect_to_terminal(&mut self) {
        let scanner = std::io::stdin();
        let mut line = String::new();
        loop {
            line.clear();
            scanner.read_line(&mut line).unwrap();
            let cmd = line.trim();
            self.receive_cmd(cmd);
        }
    }
}

impl Uci{
    pub fn send_info(result: &SearchResult){
        println!("info depth {} score cp {} nodes {} q_nodes {} nps {} time {} bestmove {} pv {}", 
            result.depth,
            result.eval,
            result.nodes,
            result.q_nodes,
            (result.nodes + result.q_nodes) as u128 * 1000 / (u128::max(1, result.timer_elapsed())),
            result.timer_elapsed(),
            result.best_move.to_str(),
            result.pv.iter().map(|a| a.to_str()).collect::<Vec<String>>().join(" ")
        );
    }
}