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
        NotationUtil::try_turn_from_notation(notation_move).unwrap_or_else(|| {
            panic!("Invalid chess move notation: Must be in standard algebraic format. Found: '{}'", notation_move)
        })
    }

    /// Converts a UCI move ("e2e4", "e7e8r") to a `Turn`, or `None` if it is not well-formed.
    /// The promotion letter is lower case as UCI prescribes (`q`, `r`, `b`, `n`) and only
    /// allowed on a move to the first or last rank.
    pub fn try_turn_from_notation(notation_move: &str) -> Option<Turn> {
        let valid_move_regex = Regex::new(r"^[a-h][1-8][a-h][1-8][qrbn]?$").unwrap();
        if !valid_move_regex.is_match(notation_move) {
            return None;
        }

        let from = NotationUtil::get_index_from_notation_field(&notation_move[0..2]);
        let to = NotationUtil::get_index_from_notation_field(&notation_move[2..4]);
        let mut promotion = 0;

        if let Some(piece) = notation_move.chars().nth(4) {
            let white_piece = match piece {
                'r' => 11,
                'n' => 12,
                'b' => 13,
                _ => 14, // 'q', the regex admits nothing else
            };
            promotion = match notation_move.chars().nth(3) {
                Some('8') => white_piece,
                Some('1') => white_piece + 10, // black piece
                _ => return None,
            };
        }
        Some(Turn::new(from, to, 0, promotion, 0, false))
    }

    /// Finds the move written as `notation` in `move_list`, matching squares and promotion piece.
    pub fn find_turn_in_list(move_list: &[Turn], notation: &str) -> Option<Turn> {
        let target = NotationUtil::try_turn_from_notation(notation)?;
        move_list.iter()
            .find(|turn| turn.from == target.from && turn.to == target.to && turn.promotion == target.promotion)
            .cloned()
    }

    /// Return a long algebraic notation. Also known as SAN
    /// Call it after the move has been made: the piece letter is read from the target square.
    /// A promotion is written as "e7e8=R"; `Pgn::set_moves` turns that into "e8=R".
    pub fn get_long_algebraic(move_notation: &str, board: &Board) -> String {
        let turn = NotationUtil::get_turn_from_notation(move_notation);
        if turn.is_promotion() {
            return format!("{}={}", &move_notation[0..4], move_notation[4..5].to_uppercase());
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_conversion() {
        assert_eq!(NotationUtil::get_index_from_notation_field("e2"), 85);
        assert_eq!(NotationUtil::get_index_from_notation_field("d6"), 44);
        assert_eq!(NotationUtil::get_index_from_notation_field("a1"), 91);
        assert_eq!(NotationUtil::get_index_from_notation_field("h8"), 28);
    }

    #[test]
    #[should_panic(expected = "Invalid chess move notation")]
    fn test_invalid_notation_panics() {
        NotationUtil::get_turn_from_notation("e2e9");
    }

    #[test]
    #[should_panic(expected = "Invalid chess move notation")]
    fn test_out_of_bounds_notation_panics() {
        NotationUtil::get_turn_from_notation("z9z9");
    }

    #[test]
    fn test_get_turn_from_notation() {
        let turn = NotationUtil::get_turn_from_notation("e2e4");
        assert_eq!(turn.from, 85);
        assert_eq!(turn.to, 65);
        assert_eq!(turn.promotion, 0);
    }

    #[test]
    fn test_promotion_to_each_piece() {
        for (suffix, white, black) in [("q", 14, 24), ("r", 11, 21), ("b", 13, 23), ("n", 12, 22)] {
            assert_eq!(NotationUtil::get_turn_from_notation(&format!("a7a8{}", suffix)).promotion, white, "a7a8{}", suffix);
            assert_eq!(NotationUtil::get_turn_from_notation(&format!("h2g1{}", suffix)).promotion, black, "h2g1{}", suffix);
        }
    }

    #[test]
    fn test_turn_to_algebraic_round_trip() {
        for notation in ["e2e4", "e1g1", "e7e8q", "e7e8r", "e7e8b", "e7e8n", "a2b1q", "a2b1r", "a2b1b", "a2b1n"] {
            assert_eq!(NotationUtil::get_turn_from_notation(notation).to_algebraic(), notation);
        }
    }
}

