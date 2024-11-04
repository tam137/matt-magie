use crate::model::Turn;
use crate::model::Board;
use regex::Regex;

pub struct NotationUtil;

impl NotationUtil {
    /// Converts a notation field (like "e2") to an index on the 10x12 board.
    pub fn get_index_from_notation_field(notation: &str) -> i32 {
        let col = match notation.chars().nth(0) {
            Some('a') => 1,
            Some('b') => 2,
            Some('c') => 3,
            Some('d') => 4,
            Some('e') => 5,
            Some('f') => 6,
            Some('g') => 7,
            Some('h') => 8,
            _ => -1
        };
        let row = 10 - notation.chars().nth(1).unwrap().to_digit(10).unwrap_or(0) as i32;
        (row * 10) + col
    }

    /// Converts a notation move (like "e2e4") to a `Turn` object.
    pub fn get_turn_from_notation(notation_move: &str) -> Turn {

        let valid_move_regex = Regex::new(r"^[a-h][1-8][a-h][1-8][qkbnr]?$").unwrap();
        if !valid_move_regex.is_match(notation_move) {
            panic!("Invalid chess move notation: Must be in standard algebraic format. Found: '{}'", notation_move);
        }

        let from = NotationUtil::get_index_from_notation_field(&notation_move[0..2]);
        let to = NotationUtil::get_index_from_notation_field(&notation_move[2..4]);
        let mut promotion = 0;

        // Promotion logic for white
        if notation_move.len() == 5 && notation_move.chars().nth(3) == Some('8') {
            promotion = match notation_move.chars().nth(4) {
                Some('n') => 12,
                _ => 14, // default to queen
            };
        }

        // Promotion logic for black
        if notation_move.len() == 5 && notation_move.chars().nth(3) == Some('1') {
            promotion = match notation_move.chars().nth(4) {
                Some('n') => 22,
                _ => 24, // default to queen
            };
        }
        Turn::new(from, to, 0, promotion, 0, false)
    }

    /// Finds a specific move in the move list based on the notation.
    pub fn get_turn_from_list(move_list: &Vec<Turn>, notation: &str) -> Turn {
        let mut target_turn = NotationUtil::get_turn_from_notation(notation);

        // Handle promotion
        if notation.len() == 5 {
            match notation.chars().nth(4) {
                Some('q') => target_turn.promotion = 14,
                Some('n') => target_turn.promotion = 12,
                Some('Q') => target_turn.promotion = 14,
                Some('N') => target_turn.promotion = 12,
                _ => panic!("Invalid promotion"),
            }

            if target_turn.to / 90 == 1 {
                target_turn.promotion = target_turn.promotion + 10; // for black promotion
            }
        }

        for move_turn in move_list {
            if move_turn.from == target_turn.from
                && move_turn.to == target_turn.to
                && move_turn.promotion == target_turn.promotion
            {
                return move_turn.clone(); // Return the found move
            }
        }
        panic!("Turn not found in the move list for notation: {}", notation);
    }

    /// Return a long algebraic notation. Also known as SAN
    pub fn get_long_algebraic(move_notation: &str, board: &Board) -> String {
        let turn = NotationUtil::get_turn_from_notation(move_notation);
        let figure = board.field[turn.to as usize] % 10;
        let figure_str = match figure {
            1 => "R".to_string(),
            2 => "N".to_string(),
            3 => "B".to_string(),
            4 => "Q".to_string(),
            5 => "K".to_string(),
            _ => "".to_string(), // Pawn
        };
        format!("{}{}", figure_str, move_notation)
    }
}

