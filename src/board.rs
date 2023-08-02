
use std::collections::HashMap;
use crate::Turn;

static TARGETS_FOR_SHORT_WHITE: [i32; 3] = [95, 96, 97];
static TARGETS_FOR_LONG_WHITE: [i32; 3] = [95, 94, 93];
static TARGETS_FOR_SHORT_BLACK: [i32; 3] = [25, 26, 27];
static TARGETS_FOR_LONG_BLACK: [i32; 3] = [25, 24, 23];

#[derive(Clone)]
pub struct Board {
    field: [i32; 120],
    pty: u32,
    fifty_move_rule: u32,
    state: GameState,
    moves: String,
    turns: Vec<Turn>,
    position_map: HashMap<String, i32>,
}

#[derive(PartialEq)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(Debug)]
pub enum GameState {
    Draw,
    WhiteWin,
    BlackWin,
    Normal,
    WhiteWinByTime,
    BlackWinByTime,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.field == other.field
    }
}


pub fn get_index_from_notation(notation: &str) -> Option<usize> {
    let chars: Vec<char> = notation.chars().collect();
    if chars.len() != 2 {
        return None;
    }
    let col = match chars[0] {
        'a' => 1,
        'b' => 2,
        'c' => 3,
        'd' => 4,
        'e' => 5,
        'f' => 6,
        'g' => 7,
        'h' => 8,
        _ => return None,
    };
    let row = match chars[1].to_digit(10) {
        Some(digit) => 10 - digit as usize,
        None => return None,
    };
    if row < 2 || row > 9 {
        return None;
    }
    Some((row * 10) + col)
}


impl Board {

    pub fn new() -> Board {

        Board {
            field: [
                -11, -11, -11, -11, -11, -11, -11, -11, -11, -11,
                -11, -11, -11, -11, -11, -11, -11, -11, -11, -11,
                //   a   b   c   d   e   f   g   h
                -11, 21, 22, 23, 24, 25, 23, 22, 21, -11, //20 - 8
                -11, 20, 20, 20, 20, 20, 20, 20, 20, -11, //30 - 7
                -11,  0,  0,  0,  0,  0,  0,  0,  0, -11, //40 - 6
                -11,  0,  0,  0,  0,  0,  0,  0,  0, -11, //50 - 5
                -11,  0,  0,  0,  0,  0,  0,  0,  0, -11, //60 - 4
                -11,  0,  0,  0,  0,  0,  0,  0,  0, -11, //70 - 3
                -11, 10, 10, 10, 10, 10, 10, 10, 10, -11, //80 - 2
                -11, 11, 12, 13, 14, 15, 13, 12, 11, -11, //90 - 1
                //    1   2   3   4   5   6   7   8 <- Indexbezeichnungen
                -11, -11, -11, -11, -11, -11, -11, -11, -11, -11,
                -11, -11, -11, -11, -11, -11, -11, -11, -11, -11,
            ],
            pty: 0,
            fifty_move_rule: 0,
            state: GameState::Normal,
            moves: String::new(),
            turns: Vec::with_capacity(200),
            position_map: HashMap::new(),
        }
    }

    pub fn get_pty(&self) -> u32 {
        self.pty
    }

    pub fn get_state(&self) -> &GameState {
        &self.state
    }

    pub fn set_state(&mut self, state: GameState) {
        self.state = state;
    }

    pub fn get_field(&self) -> &[i32; 120] {
        &self.field
    }

    pub fn set_field_index(&mut self, index: usize, piece: i32) {
        self.field[index] = piece;
    }

    pub fn clear_field(&mut self) {
        for i in 21..99 {
            if self.field[i] > 0 { self.field[i] = 0 };
        }
    }


    pub fn get_turn_list(&mut self, white: bool, only_capture: bool) -> Vec<Turn> {
        let moves = self.generate_moves_list(white);
        let mut turn_list = Vec::with_capacity(50);
        let mut last_from: usize = 0;
        let mut last_to: usize;
        for (i, &mv) in moves.iter().enumerate() {
            if i % 2 == 0 {
                last_from = mv;
            } else {
                last_to = mv;
                let capture = self.field[last_to] as i8;
                if only_capture && capture == 0 { continue }
                turn_list.push(Turn {
                    from: last_from,
                    to: last_to,
                    capture,
                    post_villain:  Vec::new(),
                    post_my: Vec::new(),
                    promotion: false,
                });
            }
        }

        for turn in &mut turn_list {
            turn.enrich_promotion_move(self, white);
        }

        for turn in turn_list.iter_mut() {
            self.do_turn(turn);
            turn.post_villain = self.generate_moves_list(!self.is_white_field(turn.to));
            let prune: bool = self.prune_illegal_moves(turn);
            if prune { self.do_undo_turn(turn); continue }
            turn.post_my = self.generate_moves_list(self.is_white_field(turn.to));            
            self.do_undo_turn(turn);
        }
        
        turn_list.retain(|turn| !turn.post_my.is_empty());
        
        if turn_list.len() == 0 { 
            if self.is_in_chess(&Board::get_target_fields_of_raw_moves(&self.generate_moves_list(!white)), white) {
                self.state = if white { GameState::BlackWin } else { GameState::WhiteWin }
            } else {
                self.state = GameState::Draw;
            }
        }
        self.state = if self.position_map.values().any(|&value| value > 2) { GameState::Draw } else { self.state };
        turn_list
    }


    pub fn do_turn(&mut self, turn: &Turn) {
        if self.field[turn.from] % 10 == 0 || self.field[turn.to] != 0 { self.fifty_move_rule = 0 } else { self.fifty_move_rule += 1 };
        self.validate_turn(turn);
        if turn.from == 95 || turn.from == 25 {
            if      turn.from == 95 && turn.to == 97 && self.field[turn.from] == 15 && self.field[96] == 0 { self.field[98] = 0;  self.field[96] = 11; }
            else if turn.from == 95 && turn.to == 93 && self.field[turn.from] == 15 && self.field[94] == 0 && self.field[93] == 0 && self.field[92] == 0 { self.field[91] = 0;  self.field[94] = 11; }
            else if turn.from == 25 && turn.to == 27 && self.field[turn.from] == 25 && self.field[26] == 0 { self.field[28] = 0;  self.field[26] = 21; }
            else if turn.from == 25 && turn.to == 23 && self.field[turn.from] == 25 && self.field[24] == 0 && self.field[23] == 0 && self.field[22] == 0 { self.field[21] = 0;  self.field[24] = 21; }
        }
        if turn.is_promotion() {
            self.field[turn.to] = if self.is_white_field(turn.from) { 14 } else { 24 };
        } else {
            self.field[turn.to] = self.field[turn.from];
        }
        self.field[turn.from] = 0;
        self.pty += 1;
        self.moves += " ";
        self.moves += &turn.to_algebraic(false).clone();
        self.turns.push(turn.clone());
    }


    pub fn do_turn_and_return_long_algebraic(&mut self, turn: &Turn) -> String {
        let figure_sign = self.get_piece_for_field(turn.from);
        let long_algebraic = format!("{}{}", figure_sign, turn.to_algebraic(true));
        self.do_turn(turn);
        long_algebraic
    }


    pub fn do_undo_turn(&mut self, turn: &Turn) {
        if turn.from == 95 || turn.from == 25 {
            if      turn.from == 95 && turn.to == 97 && self.field[turn.to] == 15 { self.field[98] = 11;  self.field[96] = 0; }
            else if turn.from == 95 && turn.to == 93 && self.field[turn.to] == 15 { self.field[91] = 11;  self.field[94] = 0; }
            else if turn.from == 25 && turn.to == 27 && self.field[turn.to] == 25 { self.field[28] = 21;  self.field[26] = 0; }
            else if turn.from == 25 && turn.to == 23 && self.field[turn.to] == 25 { self.field[21] = 21;  self.field[24] = 0; }
        }
        if turn.is_promotion() {
            self.field[turn.from] = if self.is_white_field(turn.to) { 10 } else  { 20 };
        } else {
            self.field[turn.from] = self.field[turn.to];
        }
        self.field[turn.to] = if turn.capture == -1 { 0 } else { turn.capture as i32 };
        self.pty -= 1;
        self.state = GameState::Normal;
        self.moves = self.moves[..self.moves.len() - 5].to_string();
        self.turns.remove(self.turns.len() - 1);
    }


    pub(crate) fn prune_illegal_moves(&self, turn: &mut Turn) -> bool {
        let villains_target_fields = &Board::get_target_fields_of_raw_moves(&turn.post_villain);
        let white = self.is_white_field(turn.to);
        if self.is_in_chess(villains_target_fields, white) { return true }
        match (turn.from, turn.to) {
            (95, 97) if self.field[turn.to] == 15 && TARGETS_FOR_SHORT_WHITE.iter().any(|&target| villains_target_fields.contains(&target)) => return true,
            (95, 93) if self.field[turn.to] == 15 && TARGETS_FOR_LONG_WHITE.iter().any(|&target| villains_target_fields.contains(&target)) => return true,
            (25, 27) if self.field[turn.to] == 25 && TARGETS_FOR_SHORT_BLACK.iter().any(|&target| villains_target_fields.contains(&target)) => return true,
            (25, 23) if self.field[turn.to] == 25 && TARGETS_FOR_LONG_BLACK.iter().any(|&target| villains_target_fields.contains(&target)) => return true,
            _ => return false,
        }
    }


    pub(crate) fn get_target_fields_of_raw_moves(raw_moves: &Vec<usize>) -> Vec<i32> {
        let mut villains_target_fields: Vec<i32> = Vec::with_capacity(60);
        for (i, num) in raw_moves.iter().enumerate() {
            if i % 2 == 1 {
                villains_target_fields.push(*num as i32);
            }
        }
        villains_target_fields
    }


    pub(crate) fn is_in_chess(&self, villains_target_fields: &Vec<i32>, white: bool) -> bool {
        let idx_of_king = if white { self.index_of_white_king() } else  { self.index_of_black_king() };
        if villains_target_fields.contains(&idx_of_king) { return true }
        else { false }
    }


    pub(crate) fn index_of_white_king(&self) -> i32 {
        self.field.iter().position(|&x| x == 15).unwrap() as i32
    }

    pub(crate) fn index_of_black_king(&self) -> i32 {
        self.field.iter().position(|&x| x == 25).unwrap() as i32
    }


    pub(crate) fn validate_turn(&self, turn: &Turn) {
        if self.field[turn.from] < 10 { panic!("turn.from points not to a piece ({} {})", self.moves, turn.to_algebraic(true)) };
        if self.field[turn.to] != 0 && turn.capture == 0 { panic!("turn.to points not to an empty field") };
        if turn.capture != -1 && (self.field[turn.to] == 0 && turn.capture != 0) { panic!("turn.to is expected to capture") };
        if self.field[turn.to] < 0 { panic!("turn.to points not no a valid field") };
    }


    pub fn is_white_field(&self, field_index: usize) -> bool {
        if self.field[field_index] < 10 || self.field[field_index] > 25 { panic!("Can not determine turn color [{}] index:{}", self.moves, field_index) }
        if self.field[field_index] / 10 == 1 { true } else { false }
    }


    pub fn set_fen(&mut self, fen: &str) {
        self.clear_field();
    
        let mut index = 21;
        
        for c in fen.chars() {
            if c == ' ' {
                break; // Stop processing FEN string once we reach the end of the board position section
            }
            if c == '/' {
                index += 2;
            } else if c.is_digit(10) {
                index += c.to_digit(10).unwrap() as usize;
            } else {
                let piece = match c {
                    'K' => 15,
                    'Q' => 14,
                    'R' => 11,
                    'B' => 13,
                    'N' => 12,
                    'P' => 10,
                    'k' => 25,
                    'q' => 24,
                    'r' => 21,
                    'b' => 23,
                    'n' => 22,
                    'p' => 20,
                    _ => 0, // Ignore invalid characters
                };
                if piece != 0 {
                    self.field[index] = piece; // Place the piece on the board
                }
                index += 1;
            }            
        }
    }


    pub fn get_fen(&self) -> String {
        let mut fen = String::new();
        let mut empty_count = 0;

        for i in 21..99 {
            if i % 10 == 0 { continue }
            let piece = self.field[i];
            if self.field[i] == -11 {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen.push('/');
                continue;
            }
            if piece != 0 {
                if empty_count > 0 {
                    fen.push_str(&empty_count.to_string());
                    empty_count = 0;
                }
                fen.push(match piece {
                    10 => 'P',
                    11 => 'R',
                    12 => 'N',
                    13 => 'B',
                    14 => 'Q',
                    15 => 'K',
                    20 => 'p',
                    21 => 'r',
                    22 => 'n',
                    23 => 'b',
                    24 => 'q',
                    25 => 'k',
                    _ => 'x', // Placeholder for invalid pieces
                });
            } else {
                empty_count += 1;
            }
        }
        if empty_count != 0 { fen.push_str(&empty_count.to_string()) }
        fen
    }


    pub fn add_position_for_3_move_repetition_check(&mut self, fen: String) {
        *self.position_map.entry(fen).or_insert(0) += 1;
    }


    pub fn get_complexity(&self) -> i32 {
        ((self.generate_moves_list(true).len() / 2) + (self.generate_moves_list(false).len() / 2)) as i32 / 10
    }


    pub fn get_pieces_on_field(&self) -> i32 {
        self.field.iter().filter(|&x| *x > 1).count() as i32
    }

    pub fn get_piece_for_field(&self, field_nr: usize) -> &str {
        let figure = self.get_field()[field_nr] % 10;
        match figure {
            1 => "R",
            2 => "N",
            3 => "B",
            4 => "Q",
            5 => "K",
            _ => "", // Pawn
        }
    }


    pub fn get_all_made_turns(&self) -> &Vec<Turn> {
        return &self.turns;
    }

    pub fn get_last_turn(&self) -> &Turn {
        return &self.turns[self.turns.len() - 1];
    }


    pub fn generate_moves_list(&self, white: bool) -> Vec<usize> {
        let king_value = if white { 15 } else { 25 };
        let queen_value = if white { 14 } else { 24 };
        let rook_value = if white { 11 } else { 21 };
        let bishop_value = if white { 13 } else { 23 };
        let knight_value = if white { 12 } else { 22 };
        let pawn_value = if white { 10 } else { 20 };

        let field = &self.field;
        let mut moves = Vec::with_capacity(64);       
    
        for i in 21..99 {
            if field[i] <= 0 { continue; }
            if field[i] >= 10 && field[i] <= 15 && !white { continue; }
            if field[i] >= 20 && field[i] <= 25 && white { continue; }

            if field[i] == king_value {
                for &offset in &[-11, -10, -9, -1, 1, 9, 10, 11] {
                    let target = (i as i32 + offset) as usize;
                    if (field[target] == 0 || field[target] / 10 == if white { 2 } else { 1 }) && field[target] != -11 {
                        moves.push(i);
                        moves.push(target);
                    }
                }                
                if i == 95 {
                    if !self.turns.iter().any(|t| t.from == 95) {
                        if !self.turns.iter().any(|t| t.from == 98) {
                            if field[96] == 0 && field[97] == 0 && field[98] == 11 {
                                moves.push(i);
                                moves.push(i + 2);
                            }
                        }
                        if !self.turns.iter().any(|t| t.from == 91) {
                            if field[94] == 0 && field[93] == 0 && field[92] == 0 && field[91] == rook_value {
                                moves.push(i);
                                moves.push(i - 2);
                            }
                        }
                    }
                }
                if i == 25 {
                    if !self.turns.iter().any(|t| t.from == 25) {
                        if !self.turns.iter().any(|t| t.from == 28) {
                            if field[26] == 0 && field[27] == 0 && field[28] == 21 {
                                moves.push(i);
                                moves.push(i + 2);
                            }
                        }
                        if !self.turns.iter().any(|t| t.from == 21) {
                            if field[24] == 0 && field[23] == 0 && field[22] == 0 && field[21] == rook_value {
                                moves.push(i);
                                moves.push(i - 2);
                            }
                        }
                    }
                }
            }

            if field[i] == pawn_value {
                if white {
                    if field[i - 10] == 0 {
                        moves.push(i);
                        moves.push(i - 10);
                        if i >= 81 && i <= 88 && field[i - 20] == 0 {
                            moves.push(i);
                            moves.push(i - 20);
                        }
                    }
                    if field[i - 9] >= 20 {
                        moves.push(i);
                        moves.push(i - 9);
                    }
                    if field[i - 11] >= 20 {
                        moves.push(i);
                        moves.push(i - 11);
                    }
                } else {
                    if field[i + 10] == 0 {
                        moves.push(i);
                        moves.push(i + 10);
                        if i >= 31 && i <= 38 && field[i + 20] == 0 {
                            moves.push(i);
                            moves.push(i + 20);
                        }
                    }
                    if field[i + 9] < 20 && field[i + 9] > 0 {
                        moves.push(i);
                        moves.push(i + 9);
                    }
                    if field[i + 11] < 20 && field[i + 11] > 0 {
                        moves.push(i);
                        moves.push(i + 11);
                    }
                }
            }

            if field[i] == knight_value {
                for &offset in &[-21, -19, -12, -8, 8, 12, 19, 21] {
                    let target = (i as i32 + offset) as usize;
                    if field[target] == 0 || field[target] / 10 == if white { 2 } else { 1 } && field[target] != -11 {
                        moves.push(i);
                        moves.push(target);
                    }
                }
            }

            if field[i] == bishop_value {
                for &offset in &[-11, -9, 9, 11] {
                    let mut target = (i as i32 + offset) as usize;
                    while field[target] == 0 || field[target] / 10 == if white { 2 } else { 1 } {
                        moves.push(i);
                        moves.push(target);
                        if field[target] != 0 { break }
                        target = (target as i32 + offset) as usize;
                    }
                }
            }

            if field[i] == queen_value {
                for &offset in &[-11, -10, -9, -1, 1, 9, 10, 11] {
                    let mut target = (i as i32 + offset) as usize;
                    while (field[target] == 0  || field[target] / 10 == if white { 2 } else { 1 }) && field[target] != -11 {
                        moves.push(i);
                        moves.push(target);
                        if field[target] != 0 { break }
                        target = (target as i32 + offset) as usize;
                    }
                }
            }

            if field[i] == rook_value {
                for &offset in &[-10, 10, -1, 1] {
                    let mut target = (i as i32 + offset) as usize;
                    while (field[target] == 0  || field[target] / 10 == if white { 2 } else { 1 }) && field[target] != -11 {
                        moves.push(i);
                        moves.push(target);
                        if field[target] != 0 { break }
                        target = (target as i32 + offset) as usize;                        
                    }
                }
            }

        }
        moves
    }
    

}