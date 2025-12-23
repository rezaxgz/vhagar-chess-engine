use crate::core::{Board, perft::start_perft, perft_test:: test_perft};
const ENGINENAME: &str = "Vhagar";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "Reza Ghazavi";
pub struct Uci {
    board: Board,
    position_cmd: String,
}

impl Uci {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            position_cmd: String::from("position startpos moves"),
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

    fn id(&self) {
        println!("id name {} {}", ENGINENAME, VERSION);
        println!("id author {}", AUTHOR);
    }
    fn uciok(&self) {
        self.id();
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
