use crate::model::Board;
use std::collections::HashMap;
use rand::{RngExt, rngs::StdRng, SeedableRng};


const BOARD_SIZE: usize = 120;
const NUM_PIECES: usize = 12;
const FIG: [usize; 12] = [10, 11, 12, 13, 14, 15, 20, 21, 22, 23, 24, 25];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ZobristTable {
    table: [[u64; NUM_PIECES]; BOARD_SIZE],
    fig_map: HashMap<usize, usize>,
    white_to_move: u64,
    // White short, white long, black short, black long.
    castling: [u64; 4],
    // Per file, only while a pawn can take en passant.
    en_passant: [u64; 8],
    hash_map: HashMap<u64, i16>,
    entries: u32,
}

#[allow(dead_code)]
impl ZobristTable {
    pub(crate) fn new() -> Self {

        let mut rng = StdRng::seed_from_u64(137);
        let mut table = [[0u64; NUM_PIECES]; BOARD_SIZE];

        let mut fig_map = HashMap::new();
        fig_map.insert(10, 0);
        fig_map.insert(11, 1);
        fig_map.insert(12, 2);
        fig_map.insert(13, 3);
        fig_map.insert(14, 4);
        fig_map.insert(15, 5);
        fig_map.insert(20, 6);
        fig_map.insert(21, 7);
        fig_map.insert(22, 8);
        fig_map.insert(23, 9);
        fig_map.insert(24, 10);
        fig_map.insert(25, 11);

        for i in 0..BOARD_SIZE {
            for j in FIG {
                let fig_index = fig_map.get(&(j as usize)).unwrap();
                table[i][*fig_index] = rng.random();
            }
        }
        let white_to_move = rng.random();
        // drawn after the older keys, so those stay as they were
        let castling = std::array::from_fn(|_| rng.random());
        let en_passant = std::array::from_fn(|_| rng.random());
        Self { table, fig_map, white_to_move, castling, en_passant, hash_map: HashMap::with_capacity(1000), entries: 0 }
    }

    pub fn gen_hash(&self, board: &Board) -> u64 {
        let mut hash = 0u64;
        if board.white_to_move {
            hash ^= self.white_to_move;
        }
        for i in 0..BOARD_SIZE {
            let piece = board.field[i];
            if piece > 0 {
                let piece_index = self.fig_map.get(&(piece as usize)).unwrap();
                hash ^= self.table[i][*piece_index];
            }
        }
        // FIDE: positions with other castling rights or en passant chances are other positions
        let rights = [board.white_possible_to_castle_short, board.white_possible_to_castle_long,
            board.black_possible_to_castle_short, board.black_possible_to_castle_long];
        for (key, right) in self.castling.iter().zip(rights) {
            if right {
                hash ^= key;
            }
        }
        if let Some(file) = Self::en_passant_file(board) {
            hash ^= self.en_passant[file];
        }
        hash
    }

    /// The file of the en passant square when a pawn of the side to move stands beside the pawn that
    /// just made its double step (pseudo-legal: a pin is not checked).
    fn en_passant_file(board: &Board) -> Option<usize> {
        let target = board.field_for_en_passante;
        if target <= 0 {
            return None;
        }
        // White's double step leaves its target on row 7 and the pawn on row 6; Black's on row 4 and row 5.
        let (pawn, taker) = if target / 10 == 7 { (target - 10, 20) } else { (target + 10, 10) };
        let beside = |sq: i32| board.field[sq as usize] == taker;
        if beside(pawn - 1) || beside(pawn + 1) {
            Some((target % 10 - 1) as usize)
        } else {
            None
        }
    }

    pub fn get_eval_for_hash(&self, hash: &u64) -> Option<&i16> {
        self.hash_map.get(hash)
    }

    pub fn set_new_hash(&mut self, hash: &u64, eval: i16) {
        self.entries += 1;
        self.hash_map.insert(*hash, eval);
    }

    pub fn reset_hash(&mut self) {
        self.hash_map.clear();
    }
}