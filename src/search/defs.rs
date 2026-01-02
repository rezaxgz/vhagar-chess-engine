use std::time::{Duration, Instant};

use crate::{core::{Color, Piece, r#move::{Move, MoveUtil}}, search::moves::MoveType};


pub const MATE_SCORE: i16 = -30000;
const MATE_THRESHOLD: i16 = MATE_SCORE + 100;

pub type Depth = i8;
pub type Score = i16;
#[derive(Clone)]
pub struct SearchResult {
    pub eval: i16,
    pub best_move: Move,
    pub depth: i8,
    pub duration: Duration,
    pub mate: u8,      // mate in X moves
    pub nodes: u64,    // nodes searched
    pub q_nodes: u64,   // quiescence nodes searched
    pub beta_cuttoffs: u64,
    pub tt_hits: u64,
    pub pv: Vec<Move>, // Principal Variation
    pub start_time: Instant,
}
impl SearchResult {
    pub fn stalemate() -> Self {
        return SearchResult {
            best_move: 0,
            depth: 0,
            duration: Duration::from_nanos(1),
            eval: 0,
            mate: 0,
            nodes: 1,
            pv: vec![],
            beta_cuttoffs: 0,
            tt_hits: 0,
            q_nodes: 0,
            start_time: Instant::now(),
        };
    }
    pub fn checkmate() -> Self {
        return SearchResult {
            best_move: 0,
            depth: 0,
            duration: Duration::from_nanos(1),
            eval: MATE_SCORE,
            mate: 1,
            nodes: 1,
            pv: vec![],
            beta_cuttoffs: 0,
            tt_hits: 0,
            q_nodes: 0,
            start_time: Instant::now(),
        };
    }
    pub fn inital() -> Self {
        return SearchResult {
            best_move: 0,
            depth: 0,
            duration: Duration::from_nanos(1),
            eval: Score::MIN + 10,
            mate: 0,
            nodes: 0,
            pv: vec![],
            beta_cuttoffs: 0,
            tt_hits: 0,
            q_nodes: 0,
            start_time: Instant::now(),
        };
    }
    pub fn update_stats(&mut self, thread_data: &ThreadData){
        self.beta_cuttoffs += thread_data.beta_cutoffs;
        self.nodes += thread_data.nodes;
        self.q_nodes += thread_data.q_nodes;
        self.tt_hits += thread_data.tt_hits;
        self.depth = thread_data.depth;
    }
    pub fn timer_elapsed(&self) -> u128{
        return self.start_time.elapsed().as_millis()
    }
    pub fn update(&mut self, other: &SearchResult) -> SearchResult{
        if other.eval > self.eval {
            self.set_eval(other.eval);
            self.best_move = other.best_move;
        }
        return self.clone();
    }
    pub fn set_eval(&mut self, eval: i16){
        self.eval = eval;
        if eval <= MATE_THRESHOLD {
            self.mate = ((eval - MATE_SCORE) / 2) as u8;
        } else if eval >= -MATE_THRESHOLD {
            self.mate = ((MATE_SCORE + eval) / 2) as u8;
        } else {
            self.mate = 0;
        }
    }
    pub fn update_timer(&mut self){
        self.duration = self.start_time.elapsed();
    }
}


#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    MoveTime, // Run until 'time per move' is used up.
    GameTime, // Search determines when to quit, depending on available time.
    Infinite, // Run forever, until the 'stop' command is received.
    Nothing,  // No search mode has been defined.
}

#[derive(PartialEq, Copy, Clone)]
pub struct GameTime {
    pub wtime: u128,                // White time on the clock in milliseconds
    pub btime: u128,                // Black time on the clock in milliseconds
    pub winc: u128,                 // White time increment in milliseconds (if wtime > 0)
    pub binc: u128,                 // Black time increment in milliseconds (if btime > 0)
    pub moves_to_go: Option<usize>, // Moves to go to next time control (0 = sudden death)
}
impl GameTime {
    pub fn new(
        wtime: u128,
        btime: u128,
        winc: u128,
        binc: u128,
        moves_to_go: Option<usize>,
    ) -> Self {
        return GameTime {
            binc,
            btime,
            moves_to_go,
            winc,
            wtime,
        };
    }
}
#[derive(PartialEq, Copy, Clone)]
pub struct SearchInfo {
    //info relating to the current state of the search
    pub search_mode: SearchMode, // Defines the mode to search in

    //limits applied to the search
    pub allocated_time: u128, // Allotted msecs to spend on move (soft bound)
    pub max_depth: i8,        // Maximum depth to search to
    pub max_move_time: u128,  // Maximum time per move to search (hard bound)
    pub max_nodes: u64,       // Maximum number of nodes to search
    pub game_time: GameTime,  // Time available for entire game

    //info relating to search summary
    pub start_time: Instant,   // Time the search started
    pub quiet: bool,           // No intermediate search stats updates
}

impl SearchInfo {
    pub fn default() -> Self {
        return SearchInfo {
            start_time: Instant::now(),
            max_nodes: u64::MAX,
            allocated_time: 0,
            max_depth: i8::MAX,
            max_move_time: 60 * 1000 * 5, // 5 minutes
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
        };
    }
    pub fn timer_start(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn timer_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn is_terminated(&self, is_mid_search: bool) -> bool {
        let elapsed = self.timer_elapsed().as_millis();
        if self.search_mode == SearchMode::Infinite {
            return false;
        }
        if elapsed > self.max_move_time {
            return true;
        }
        match self.search_mode {
            SearchMode::MoveTime => return elapsed > self.allocated_time,
            SearchMode::GameTime => return elapsed > self.allocated_time && !is_mid_search,
            SearchMode::Infinite => return false,
            _ => return elapsed > self.max_move_time,
        }
    }

    pub fn reset_temp_info(&mut self) {
        
    }
}
pub const KILLERS_PER_PLY: usize = 2;
pub const KILLER_PLIES: usize = 20;
pub type KillerMoves = [Move; KILLERS_PER_PLY];
pub struct ThreadData {
    killers: [KillerMoves; 20],  
    pub history: [[[i32; 64]; 6]; 2],  // [piece][to]
    pub move_values: [i32; 65536],     // used for faster move ordering
    pub move_types: [MoveType; 65536], // used for faster move ordering

    //current search data
    pub ply: i8,                       // Number of plys from the root
    pub depth: i8,                     // Depth currently being searched

    //search summary info
    pub nodes: u64,                    // Nodes searched
    pub q_nodes: u64,                  // Number of quescence nodes
    pub tt_hits: u64,                  // Number of transposition table hits
    pub beta_cutoffs: u64,             // Number of beta cutoffs
}


impl ThreadData{
    pub fn new() -> ThreadData{
        return ThreadData { 
            killers: [[0; 2]; 20],
            history: [[[0; 64]; 6]; 2],
            move_values: [0; 65536],
            move_types: [MoveType::BadCapture; 65536],
            ply: 0, 
            depth: 0,
            nodes: 0, 
            q_nodes: 0,
            tt_hits: 0,
            beta_cutoffs: 0,
        };
    }
    pub fn get_killers(&self, ply: Depth) -> KillerMoves{
        if ply as usize >= KILLER_PLIES {
            return [0; 2];
        }
        return self.killers[ply as usize];
    }
    pub fn store_killer_move(&mut self, depth: Depth, ply: Depth, mv: Move, piece: Piece, color: Color){ 
        //do not store captures or promotions as killer moves
        self.history[color as usize][piece as usize][mv.get_to() as usize] = (self.history[color as usize][piece as usize][mv.get_to() as usize] + (depth as i32 * depth as i32)).clamp(-32_000, 32_000);
        if ply as usize >= KILLER_PLIES {
            return;
        }
        if self.killers[ply as usize][0] != mv {
            self.killers[ply as usize][1] = self.killers[ply as usize][0];
            self.killers[ply as usize][0] = mv;
        }
    }
    pub fn store_bad_quiet(&mut self, depth: Depth, mv: Move, piece: Piece, color: Color){
        self.history[color as usize][piece as usize][mv.get_to() as usize] = (self.history[color as usize][piece as usize][mv.get_to() as usize] + (depth as i32 * depth as i32)).clamp(-32_000, 32_000);
    }
}
