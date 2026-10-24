use std::collections::HashMap;

use crate::{notation_util::NotationUtil, zobrist::ZobristTable};


#[derive(Debug, PartialEq, Clone)]
pub enum GameStatus {
    Normal,
    DrawByStalemate,
    DrawByRepetition,
    // A hundred plies without a pawn move or a capture.
    DrawByFiftyMoves,
    // A dead position: K v K, a king and one minor piece against a king, or bishops all on one colour.
    DrawByInsufficientMaterial,
    // The safety cap MOVE_LIMIT, against games that would never end.
    DrawByMoveLimit,
    WhiteWin,
    BlackWin,
    WhiteWinByTime,
    BlackWinByTime,
    // The other engine sent an illegal or unreadable move (or moved out of turn).
    WhiteWinByIllegalMove,
    BlackWinByIllegalMove
}


// A game that reaches this move number is drawn (only against games that would never end).
pub const MOVE_LIMIT: i32 = 400;

pub struct UciGame {
    pub board: Board,
    pub made_moves_str: String,
    pub pty: i32,
    // Plies since the last pawn move or capture, for the fifty-move rule.
    pub halfmove_clock: i32,
}

impl UciGame {

    pub fn new(board: Board) -> Self {
        UciGame {
            board,
            made_moves_str: String::from(""),
            pty: 0,
            halfmove_clock: 0,
        }
    }

    pub fn do_move(&mut self, notation_move: &str) {
        let turn = NotationUtil::get_turn_from_notation(notation_move);
        let piece = self.board.field[turn.from as usize];
        // a pawn move (en passant included) or a capture resets the fifty-move count
        if piece == 10 || piece == 20 || self.board.field[turn.to as usize] != 0 {
            self.halfmove_clock = 0;
        } else {
            self.halfmove_clock += 1;
        }
        self.board.do_move(&turn);
        self.pty += 1;
        
        if self.made_moves_str.is_empty() {
            self.made_moves_str.push_str(notation_move);
        } else {
            self.made_moves_str.push(' ');
            self.made_moves_str.push_str(notation_move);
        }
    }

    /// Ends a game that is still running by the draw rules: insufficient material, then the
    /// fifty-move rule, then the move limit. Mate, stalemate and repetition are already set by then.
    pub fn apply_draw_rules(&mut self) {
        if self.board.game_status != GameStatus::Normal {
            return;
        }
        if self.board.is_insufficient_material() {
            self.board.game_status = GameStatus::DrawByInsufficientMaterial;
        } else if self.halfmove_clock >= 100 {
            self.board.game_status = GameStatus::DrawByFiftyMoves;
        } else if self.board.move_count > MOVE_LIMIT {
            self.board.game_status = GameStatus::DrawByMoveLimit;
        }
    }

    #[allow(dead_code)]
    pub fn white_to_move(&self) -> bool  {
        self.board.white_to_move
    }
}


#[derive(Debug, Clone)]
pub struct Turn {
    pub from: i32,
    pub to: i32,
    pub capture: i32,
    pub promotion: i32,
    pub eval: i16,
    pub gives_check: bool,
}

impl PartialEq for Turn {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
            && self.to == other.to
            && self.capture == other.capture
            && self.promotion == other.promotion
            && self.eval == other.eval
            && self.gives_check == other.gives_check
    }
}

impl PartialEq<&Turn> for Turn {
    fn eq(&self, other: &&Turn) -> bool {
        self == *other
    }
}

impl Turn {
    // Constructor with all fields
    pub fn new(from: i32, to: i32, capture: i32, promotion: i32, eval: i16, gives_check: bool) -> Self {
        Turn {
            from,
            to,
            capture,
            promotion,
            eval,
            gives_check,
        }
    }

    // Constructor with only 'from' and 'to' fields
    #[allow(dead_code)]
    pub fn from_to(from: i32, to: i32) -> Self {
        Turn {
            from,
            to,
            capture: 0,
            promotion: 0,
            eval: 0,
            gives_check: false
        }
    }

    // Check if the move is a promotion
    pub fn is_promotion(&self) -> bool {
        self.promotion != 0
    }

    // Set promotion with fluent interface
    #[allow(dead_code)]
    pub fn set_promotion(mut self, promotion: i32) -> Self {
        self.promotion = promotion;
        self
    }

    pub fn to_algebraic(&self) -> String {
        let column_from = (self.from % 10 + 96) as u8;
        let row_from = (10 - (self.from / 10) + 48) as u8;
        let column_to = (self.to % 10 + 96) as u8;
        let row_to = (10 - (self.to / 10) + 48) as u8;
        let promotional_lit = if self.promotion == 0 {
            ""
        } else {
            match self.promotion % 10 {
                1 => "r",
                2 => "n",
                3 => "b",
                _ => "q",
            }
        };
        format!(
            "{}{}{}{}{}",
            column_from as char, row_from as char, column_to as char, row_to as char, &promotional_lit
        )
    }
}



#[derive(Debug, Copy, Clone)]
pub struct MoveInformation {
    pub castle_information: CastleInformation,
    pub hash: u64,
    pub en_passante: i32,
}

impl MoveInformation {
    // Constructor
    pub fn new(castle_information: CastleInformation, hash: u64, en_passante: i32) -> Self {
        MoveInformation {
            castle_information,
            hash,
            en_passante,
        }
    }
}


#[derive(Debug, Copy, Clone)]
pub struct CastleInformation {
    pub white_possible_to_castle_long: bool,
    pub white_possible_to_castle_short: bool,
    pub black_possible_to_castle_long: bool,
    pub black_possible_to_castle_short: bool,
}

impl CastleInformation {
    // Constructor
    #[allow(dead_code)]
    pub fn new(white_possible_to_castle_long: bool, white_possible_to_castle_short: bool,
               black_possible_to_castle_long: bool, black_possible_to_castle_short: bool) -> Self {
        CastleInformation {
            white_possible_to_castle_long,
            white_possible_to_castle_short,
            black_possible_to_castle_long,
            black_possible_to_castle_short,
        }
    }
}


#[derive(Debug, Clone)]
pub struct Board {
    pub field: [i32; 120],
    pub white_possible_to_castle_long: bool,
    pub white_possible_to_castle_short: bool,
    pub black_possible_to_castle_long: bool,
    pub black_possible_to_castle_short: bool,
    pub field_for_en_passante: i32,  // 0 if no en passant possible, only used by fen import
    pub white_to_move: bool,
    pub move_count: i32,
    pub game_status: GameStatus,
    pub move_repetition_map: HashMap<u64, i32>,
    pub zobrist: ZobristTable,
}

impl Board {
    // Constructor
    pub fn new(
        field: [i32; 120],
        white_possible_to_castle_long: bool,
        white_possible_to_castle_short: bool,
        black_possible_to_castle_long: bool,
        black_possible_to_castle_short: bool,
        field_for_en_passante: i32,
        white_to_move: bool,
        move_count: i32,
        zobrist: ZobristTable,
    ) -> Self {
        Board {
            field,
            white_possible_to_castle_long,
            white_possible_to_castle_short,
            black_possible_to_castle_long,
            black_possible_to_castle_short,
            field_for_en_passante,
            white_to_move,
            move_count,
            game_status: GameStatus::Normal,
            move_repetition_map: HashMap::new(),
            zobrist,
        }
    }


    // Method for performing a move
    pub fn do_move(&mut self, turn: &Turn) -> MoveInformation {

        // validation
        if self.field[turn.from as usize] == 0 {
            panic!("do_move(): Field on turn.from is 0\n{:?}", turn);
        }
        
        let old_castle_information = self.get_castle_information();
        let old_field_for_en_passante = self.field_for_en_passante;

        // Handling en passante information
        self.field_for_en_passante = -1;
        if (self.white_to_move && turn.from / 10 == 8 && turn.to / 10 == 6 && self.field[turn.from as usize] == 10) 
            || (!self.white_to_move && turn.from / 10 == 3 && turn.to / 10 == 5 && self.field[turn.from as usize] == 20) {
            
            let base = if self.white_to_move { 70 } else { 40 };
            self.field_for_en_passante = base + (turn.from % 10);
        }

        // Handling castling for white and black
        if self.field[turn.from as usize] == 15 || self.field[turn.from as usize] == 25 {
            match (turn.from, turn.to) {
                (25, 27) => {
                    self.field[28] = 0;
                    self.field[26] = 21;
                }
                (25, 23) => {
                    self.field[21] = 0;
                    self.field[24] = 21;
                }
                (95, 97) => {
                    self.field[98] = 0;
                    self.field[96] = 11;
                }
                (95, 93) => {
                    self.field[91] = 0;
                    self.field[94] = 11;
                }
                _ => {}
            }
        }

        // Update castling rights
        match turn.from {
            21 => self.black_possible_to_castle_long = false,
            28 => self.black_possible_to_castle_short = false,
            25 => {
                self.black_possible_to_castle_long = false;
                self.black_possible_to_castle_short = false;
            }
            91 => self.white_possible_to_castle_long = false,
            98 => self.white_possible_to_castle_short = false,
            95 => {
                self.white_possible_to_castle_long = false;
                self.white_possible_to_castle_short = false;
            }
            _ => {}
        }

        // Handle promotion
        if turn.is_promotion() {
            self.field[turn.to as usize] = turn.promotion;
        } else {
            self.field[turn.to as usize] = self.field[turn.from as usize];
        }

        self.field[turn.from as usize] = 0;

        // Handle en passante
        if old_field_for_en_passante == turn.to && self.field[turn.to as usize] == 10 {
            self.field[(turn.to + 10) as usize] = 0;
        } else if old_field_for_en_passante == turn.to && self.field[turn.to as usize] == 20 {
            self.field[(turn.to - 10) as usize] = 0;
        }

        // Increment move count if it's black's turn
        if !self.white_to_move {
            self.move_count += 1;
        }
        self.white_to_move = !self.white_to_move;

        // Calculate the board hash and update the move repetition map
        let board_hash = self.hash();
        self.move_repetition_map
            .entry(board_hash)
            .and_modify(|count| *count += 1)
            .or_insert(1);

        // Check for 3-move repetition
        if let Some(&count) = self.move_repetition_map.get(&board_hash) {
            if count == 3 {
                self.game_status = GameStatus::DrawByRepetition;
            }
        }
        MoveInformation::new(old_castle_information, board_hash, old_field_for_en_passante)
    }


    /// A dead position: no pawn, rook or queen, and either at most one minor piece on the board
    /// or bishops only, all on squares of one colour.
    pub fn is_insufficient_material(&self) -> bool {
        let (mut minors, mut knights) = (0, 0);
        let mut bishop_colours = [false, false];
        for idx in 21..99 {
            match self.field[idx] {
                10 | 11 | 14 | 20 | 21 | 24 => return false,
                12 | 22 => { minors += 1; knights += 1; }
                13 | 23 => { minors += 1; bishop_colours[(idx / 10 + idx % 10) % 2] = true; }
                _ => {}
            }
        }
        minors <= 1 || (knights == 0 && !(bishop_colours[0] && bishop_colours[1]))
    }

    // Undo move
    pub fn undo_move(&mut self, turn: &Turn, move_information: MoveInformation) {

        // validation
        if self.field[turn.to as usize] == 0 {
            panic!("undo_move(): Field on turn.to is 0\n{:?}", turn);
        }

        self.game_status = GameStatus::Normal;

        let castle_information = move_information.castle_information;
        let mut is_en_passante_move = false;

        // Handle en passante undo
        if turn.to == move_information.en_passante && self.field[turn.to as usize] == 10 {
            self.field[(turn.to + 10) as usize] = 20;
            is_en_passante_move = true;
        } else if turn.to == move_information.en_passante && self.field[turn.to as usize] == 20 {
            self.field[(turn.to - 10) as usize] = 10;
            is_en_passante_move = true;
        }

        // Handle promotion undo
        if turn.is_promotion() {
            self.field[turn.from as usize] = 10; // Reset to pawn
            if self.white_to_move {
                self.field[turn.from as usize] += 10; // Black pawn for black promotion
            }
        } else {
            self.field[turn.from as usize] = self.field[turn.to as usize];
        }

        if is_en_passante_move {
            self.field[turn.to as usize] = 0;
        } else {
            self.field[turn.to as usize] = turn.capture.max(0);
        }        

        // Restore castling rights and en passante information
        self.white_possible_to_castle_long = castle_information.white_possible_to_castle_long;
        self.white_possible_to_castle_short = castle_information.white_possible_to_castle_short;
        self.black_possible_to_castle_long = castle_information.black_possible_to_castle_long;
        self.black_possible_to_castle_short = castle_information.black_possible_to_castle_short;
        self.field_for_en_passante = move_information.en_passante;

        // Handle castling undo
        if self.field[turn.from as usize] == 15 || self.field[turn.from as usize] == 25 {
            match (turn.from, turn.to) {
                (25, 27) => {
                    self.field[28] = 21;
                    self.field[26] = 0;
                }
                (25, 23) => {
                    self.field[21] = 21;
                    self.field[24] = 0;
                }
                (95, 97) => {
                    self.field[98] = 11;
                    self.field[96] = 0;
                }
                (95, 93) => {
                    self.field[91] = 11;
                    self.field[94] = 0;
                }
                _ => {}
            }
        }

        // Decrement move count if it was white's move
        if self.white_to_move {
            self.move_count -= 1;
        }
        self.white_to_move = !self.white_to_move;

        // Update the move repetition map
        if let Some(count) = self.move_repetition_map.get_mut(&move_information.hash) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.move_repetition_map.remove(&move_information.hash);
            }
        }
    }

    // Generate the castle information based on the current state
    pub fn get_castle_information(&self) -> CastleInformation {
        CastleInformation {
            white_possible_to_castle_long: self.white_possible_to_castle_long,
            white_possible_to_castle_short: self.white_possible_to_castle_short,
            black_possible_to_castle_long: self.black_possible_to_castle_long,
            black_possible_to_castle_short: self.black_possible_to_castle_short,
        }
    }

    // Hash function for the board (used for 3-move repetition)
    pub fn hash(&self) -> u64 {
        self.zobrist.gen_hash(&self)
    }
}

// Implement `PartialEq` manually for the `Board` struct, for unittests
impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        // Check if all the fields of the Board match
        self.white_possible_to_castle_long == other.white_possible_to_castle_long &&
            self.white_possible_to_castle_short == other.white_possible_to_castle_short &&
            self.black_possible_to_castle_long == other.black_possible_to_castle_long &&
            self.black_possible_to_castle_short == other.black_possible_to_castle_short &&
            self.field_for_en_passante == other.field_for_en_passante &&
            self.white_to_move == other.white_to_move &&
            self.move_count == other.move_count &&
            self.game_status == other.game_status &&
            self.field == other.field &&  // Direct comparison of arrays (fixed-size arrays implement PartialEq)
            self.move_repetition_map == other.move_repetition_map  // HashMap comparison
    }
}
