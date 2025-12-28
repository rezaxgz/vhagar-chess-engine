use std::cell::UnsafeCell;

use crate::{core::r#move::Move, search::defs::{Depth, Score}};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flag{
    EXACT = 0, 
    LOWER = 1, // fail-high
    UPPER = 2// fail-low
}
#[repr(C)]
#[derive(Copy, Clone)]
pub struct TTEntry {
    pub key: u16, // part of the hash
    pub best_move: Move,
    pub eval: Score,
    pub depth: Depth,
    pub flag: Flag,
    pub age: u8,
}

impl Default for TTEntry {
    fn default() -> Self {
        TTEntry {
            key: 0,
            depth: -1,
            eval: 0,
            best_move: 0,
            age: 0,
            flag: Flag::UPPER
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PawnEntry {
    pub key: u16, // part of the hash
    pub w_filesets: u8,
    pub b_filesets: u8,
    pub w_pst: (Score, Score),
    pub b_pst: (Score, Score),
    pub eval: Score,
}

impl Default for PawnEntry {
    fn default() -> Self {
        PawnEntry { key: 0, w_filesets: 0, b_filesets: 0, w_pst: (0, 0), b_pst: (0, 0), eval: 0 }
    }
}

pub struct TranspositionTable {
    table: Vec<UnsafeCell<TTEntry>>,
    mask: usize,
    pawn_table: Vec<UnsafeCell<PawnEntry>>,
    pawn_mask: usize,
}

impl TranspositionTable {
    pub fn new(mb: usize) -> Self {
        let entry_size = std::mem::size_of::<TTEntry>();
        let total_bytes = mb * 1024 * 1024;
        let pawn_bytes = total_bytes >> 4;
        let main_table_size = (total_bytes/entry_size).next_power_of_two();
        let pawn_table_size = (pawn_bytes/std::mem::size_of::<PawnEntry>());
        TranspositionTable {
            table: (0..main_table_size)
                .map(|_| UnsafeCell::new(TTEntry::default()))
                .collect(),
            mask: main_table_size - 1,
            pawn_table: (0..pawn_table_size)
                .map(|_| UnsafeCell::new(PawnEntry::default()))
                .collect(),
            pawn_mask: pawn_table_size - 1,
        }
    }
    #[inline]
    pub fn pos_index(&self, key: u64) -> usize {
        (key as usize) & self.mask
    }
    #[inline]
    pub fn pawn_index(&self, key: u64) -> usize {
        (key as usize) & self.pawn_mask
    }
    // pub fn clear(&mut self){
    //     *self = TranspositionTable {
    //         table: (0..(self.mask + 1))
    //             .map(|_| UnsafeCell::new(TTEntry::default()))
    //             .collect(),
    //         mask: self.mask,
    //         pawn_table: (0..(self.pawn_mask + 1))
    //             .map(|_| UnsafeCell::new(PawnEntry::default()))
    //             .collect(),
    //         pawn_mask: self.pawn_mask
    //     }
    // }
}

//main table
impl TranspositionTable {
    #[inline]
    pub fn lookup_position(&self, key: u64) -> Option<TTEntry> {
        let idx = self.pos_index(key);
        unsafe {
            let e = *self.table[idx].get();
            if e.key == (key >> 48) as u16 {
                Some(e)
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn store_position(&self, key: u64, entry: TTEntry) {
        let idx = self.pos_index(key);
        unsafe {*self.table[idx].get() = entry;}
    }
}

//pawn table
impl TranspositionTable {
    #[inline]
    pub fn lookup_pawn_structure(&self, key: u64) -> Option<PawnEntry> {
        let idx = self.pawn_index(key);
        unsafe {
            let e = *self.pawn_table[idx].get();
            if e.key == (key >> 48) as u16 {
                Some(e)
            } else {
                None
            }
        }
    }

    #[inline]
    pub fn store_pawn_structure(&self, key: u64, entry: PawnEntry) {
        let idx = self.pawn_index(key);
        unsafe {*self.pawn_table[idx].get() = entry;}
    }
}
unsafe impl Sync for TranspositionTable {}