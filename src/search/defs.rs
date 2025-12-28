use std::time::{Duration, Instant};

use crate::core::r#move::{ExtendedMove, Move};


pub const MATE_SCORE: i16 = -30000;
pub const MATE_THRESHOLD: i16 = MATE_SCORE + 100;
pub const DRAW_SCORE: i16 = 0;
pub const ALPHA: i16 = -i16::MAX;
pub const BETA: i16 = i16::MAX;


#[derive(Clone)]
pub struct RootResult {
    pub mv: Move,
    pub score: i16,
}

pub type Depth = i8;
pub type Score = i16;

pub struct SearchResult {
    pub eval: i16,
    pub best_move: Move,
    pub depth: i8,
    pub seldepth: i8,
    pub duration: Duration,
    pub mate: u8,      // mate in X moves
    pub nodes: u64,    // nodes searched
    pub pv: Vec<Move>, // Principal Variation
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
            seldepth: 0,
        };
    }
    pub fn checkmate() -> Self {
        return SearchResult {
            best_move: 0,
            depth: 0,
            duration: Duration::from_nanos(1),
            eval: MATE_SCORE,
            mate: 0,
            nodes: 1,
            pv: vec![],
            seldepth: 0,
        };
    }
    pub fn empty() -> Self {
        return SearchResult {
            best_move: 0,
            depth: 0,
            duration: Duration::from_nanos(1),
            eval: 0,
            mate: 0,
            nodes: 0,
            pv: vec![],
            seldepth: 0,
        };
    }
}


#[derive(PartialEq, Copy, Clone)]
pub enum SearchMode {
    Depth,    // Run until requested depth is reached.
    MoveTime, // Run until 'time per move' is used up.
    Nodes,    // Run until the number of requested nodes was reached.
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
    pub depth: i8,           // Depth currently being searched
    pub depth_extension: i8, // Depth currently extended
    pub previous_move: ExtendedMove,
    pub ply: i8,                 // Number of plys from the root
    pub search_mode: SearchMode, // Defines the mode to search in

    //limits applied to the search
    pub allocated_time: u128, // Allotted msecs to spend on move (soft bound)
    pub max_depth: i8,        // Maximum depth to search to
    pub max_move_time: u128,  // Maximum time per move to search (hard bound)
    pub max_nodes: u64,       // Maximum number of nodes to search
    pub game_time: GameTime,  // Time available for entire game

    //info relating to search summary
    pub seldepth: i8,          // Maximum selective depth reached
    pub start_time: Instant,   // Time the search started
    pub nodes: u64,            // Nodes searched
    pub tt_hits: u64,          //number of transposition table hits
    pub beta_cutoffs: u64,     // Number of beta cutoffs
    pub q_nodes: u64,          //number of quescence nodes
    pub quiet: bool,           // No intermediate search stats updates
    pub terminated_flag: bool, // If the search is terminated
}

impl SearchInfo {
    pub fn default() -> Self {
        return SearchInfo {
            start_time: Instant::now(),
            previous_move: ExtendedMove::default(),
            depth: 0,
            depth_extension: 0,
            seldepth: 0,
            nodes: 0,
            q_nodes: 0,
            beta_cutoffs: 0,
            tt_hits: 0,
            max_nodes: u64::MAX,
            ply: 0,
            allocated_time: 0,
            max_depth: i8::MAX,
            max_move_time: 60 * 1000 * 5, // 5 minutes
            game_time: GameTime::new(0, 0, 0, 0, None),
            search_mode: SearchMode::Nothing,
            quiet: false,
            terminated_flag: false,
        };
    }
    pub fn timer_start(&mut self) {
        self.start_time = Instant::now();
    }

    pub fn timer_elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    fn check_termination(&self, is_mid_search: bool) -> bool {
        let elapsed = self.timer_elapsed().as_millis();
        if self.search_mode == SearchMode::Infinite {
            return false;
        }
        if elapsed > self.max_move_time {
            return true;
        }
        match self.search_mode {
            SearchMode::MoveTime => return elapsed > self.allocated_time,
            SearchMode::Nodes => return self.nodes > self.max_nodes,
            SearchMode::Depth => return self.depth > self.max_depth,
            SearchMode::GameTime => return elapsed > self.allocated_time && !is_mid_search,
            _ => return elapsed > self.max_move_time,
        }
    }
    pub fn is_terminated(&mut self, is_mid_search: bool) -> bool {
        if self.terminated_flag {
            return true;
        }
        self.terminated_flag = self.check_termination(is_mid_search);
        return self.terminated_flag;
    }
    pub fn reset_temp_info(&mut self) {
        
    }
}
