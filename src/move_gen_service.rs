use crate::model::{Board, GameStatus, Turn};

pub struct MoveGenService {
}

impl MoveGenService {

    pub fn new() -> Self {
        MoveGenService {
        }
    }


    /// Generates a list of valid moves for a given board state.
    pub fn generate_valid_moves_list(&self, board: &mut Board) -> Vec<Turn> {
        if board.game_status != GameStatus::Normal {
            return vec![]
        }
        let move_list = self.generate_moves_list_for_piece(board, 0);
        self.get_valid_moves_from_move_list(&move_list, board)
    }

    fn get_valid_moves_from_move_list(&self, move_list: &[i32], board: &mut Board) -> Vec<Turn> {
        let mut valid_moves = Vec::with_capacity(64);
        let white_turn = board.white_to_move;
        let king_value = if white_turn { 15 } else { 25 };
    
        for i in (0..move_list.len()).step_by(2) {
            let idx0 = move_list[i];
            let idx1 = move_list[i + 1];
            let mut move_turn = Turn::new(idx0, idx1, board.field[idx1 as usize], 0, 0, false);
    
            // Check for castling
            if board.field[idx0 as usize] == king_value && (idx1 == idx0 + 2 || idx1 == idx0 - 2) {
                if !self.is_valid_castling(board, white_turn, idx1) {
                    continue;
                }
            }
    
            // Check for promotion
            if let Some(promotion_move) = self.get_promotion_move(board, white_turn, idx0, idx1) {
                move_turn.promotion = promotion_move.promotion;
                // Validate and add the promotion moves (e.g., Queen, Knight)
                self.validate_and_add_promotion_moves(board, &mut move_turn, &mut valid_moves, white_turn);
            } else {
                // Validate and add the regular move
                self.validate_and_add_move(board, &mut move_turn, &mut valid_moves, white_turn);
            }
        }
    
        // Add en passant moves
        let en_passante_turns = self.get_en_passante_turns(board, white_turn);
        for mut turn in en_passante_turns {
            self.validate_and_add_move(board, &mut turn, &mut valid_moves, white_turn);
        }
    
        if white_turn {
            valid_moves.sort_unstable_by(|a, b| b.eval.cmp(&a.eval));
        } else {
            valid_moves.sort_unstable_by(|a, b| a.eval.cmp(&b.eval));
        }
    
        // check Gamestatus
        if valid_moves.is_empty() {
            if self.get_check_idx_list(&board.field, board.white_to_move).len() > 0 {
                board.game_status = if board.white_to_move { GameStatus::BlackWin } else { GameStatus::WhiteWin }
            } else {
                board.game_status = GameStatus::Draw;
            }
        }
    
        valid_moves
    }
    
    fn get_en_passante_turns(&self, board: &Board, white_turn: bool) -> Vec<Turn> {
        let mut en_passante_turns = Vec::with_capacity(4);
        if board.field_for_en_passante != -1 {
            let target_piece = if white_turn { 20 } else { 10 };
            let offsets = if white_turn { [9, 11] } else { [-9, -11] };
            for &offset in &offsets {
                if board.field[(board.field_for_en_passante + offset) as usize] == if white_turn { 10 } else { 20 } {
                    en_passante_turns.push(
                        Turn::new(board.field_for_en_passante + offset, board.field_for_en_passante, target_piece, 0, 0, false)
                    );
                }
            }
        }
        en_passante_turns
    }
    
    fn validate_and_add_move(&self, board: &mut Board, turn: &mut Turn,valid_moves: &mut Vec<Turn>, white_turn: bool) {
        let move_info = board.do_move(turn);
        let mut valid = true;
    
        // Check if the move leads to check
        if !self.get_check_idx_list(&board.field, white_turn).is_empty() {
            valid = false;
        }
    
        // If valid, add the move to the list
        if valid {
    
            // check if the move gives opponent check
            if self.get_check_idx_list(&board.field, !white_turn).len() > 0 {
                turn.gives_check = true;
            }
            valid_moves.push(turn.clone());
        }
        board.undo_move(turn, move_info);
    }
    
    fn validate_and_add_promotion_moves(&self, board: &mut Board, turn: &mut Turn, valid_moves: &mut Vec<Turn>, white_turn: bool) {
        let promotion_types = if white_turn { [12, 14] } else { [22, 24] }; // Knight and Queen promotions for white and black
        for &promotion in &promotion_types {
            turn.promotion = promotion;
            self.validate_and_add_move(board, turn, valid_moves, white_turn);
        }
    }

    

    fn is_valid_castling(&self, board: &Board, white_turn: bool, target: i32) -> bool {
        let check_squares = if white_turn {
            if target == 97 { vec![96, 97] } else { vec![94, 93] }
        } else {
            if target == 27 { vec![26, 27] } else { vec![24, 23] }
        };

        // Check if the king is currently in check
        if !self.get_check_idx_list(&board.field, white_turn).is_empty() {
            return false;
        }

        // Check if the king would pass through check squares
        for &square in &check_squares {
            if !self.get_attack_idx_list(&board.field, white_turn, square).is_empty() {
                return false;
            }
        }

        // Check if castling is allowed
        if white_turn {
            if target == 97 && !board.white_possible_to_castle_short {
                return false;
            }
            if target == 93 && !board.white_possible_to_castle_long {
                return false;
            }
        } else {
            if target == 27 && !board.black_possible_to_castle_short {
                return false;
            }
            if target == 23 && !board.black_possible_to_castle_long {
                return false;
            }
        }
        true
    }

    fn get_promotion_move(&self, board: &Board, white_turn: bool, idx0: i32, idx1: i32) -> Option<Turn> {
        if white_turn && idx0 / 10 == 3 && board.field[idx0 as usize] == 10 {
            Some(Turn {
                from: idx0,
                to: idx1,
                capture: 0,
                promotion: 14,
                eval: 0,
                gives_check: false,
            })
        } else if !white_turn && idx0 / 10 == 8 && board.field[idx0 as usize] == 20 {
            Some(Turn {
                from: idx0,
                to: idx1,
                capture: 0,
                promotion: 24,
                eval: 0,
                gives_check: false,
            })
        } else {
            None
        }
    }


    pub fn generate_moves_list_for_piece(&self, board: &Board, idx: i32) -> Vec<i32> {
        let check_idx_list = self.get_check_idx_list(&board.field, board.white_to_move);
        let field = board.field;
        let white = board.white_to_move;

        let king_value = if white { 15 } else { 25 };
        let queen_value = if white { 14 } else { 24 };
        let rook_value = if white { 11 } else { 21 };
        let bishop_value = if white { 13 } else { 23 };
        let knight_value = if white { 12 } else { 22 };
        let pawn_value = if white { 10 } else { 20 };

        let mut moves = Vec::with_capacity(64);

        let start_idx = if idx == 0 { 21 } else { idx };
        let end_idx = if idx == 0 { 99 } else { idx + 1 };

        for i in start_idx..end_idx {
            // Skip other pieces if the king is in check from multiple pieces
            if check_idx_list.len() > 1 && field[i as usize] != king_value {
                continue;
            }

            // Skip empty squares or enemy pieces
            if field[i as usize] <= 0 {
                continue;
            }
            if (field[i as usize] >= 10 && field[i as usize] <= 15 && !white)
                || (field[i as usize] >= 20 && field[i as usize] <= 25 && white) {
                continue;
            }

            // King moves
            if field[i as usize] == king_value {
                let offsets = [-11, -10, -9, -1, 1, 9, 10, 11];
                for offset in offsets {
                    let target = i + offset;
                    if (field[target as usize] == 0 || field[target as usize] / 10 == if white { 2 } else { 1 })
                        && field[target as usize] != -11
                    {
                        let mut valid = true;
                        for target_offset in offsets {
                            if field[(target + target_offset) as usize] == if white { 25 } else { 15 } {
                                valid = false;
                                break;
                            }
                        }
                        if valid {
                            moves.push(i);
                            moves.push(target);
                        }
                    }
                }

                // Castling moves for White and Black
                if field[i as usize] == king_value {
                    if i == 95 && field[96] == 0 && field[97] == 0 && field[98] == 11 {
                        moves.push(95);
                        moves.push(97);
                    }
                    if i == 25 && field[26] == 0 && field[27] == 0 && field[28] == 21 {
                        moves.push(25);
                        moves.push(27);
                    }
                    if i == 95 && field[94] == 0 && field[93] == 0 && field[92] == 0 && field[91] == 11 {
                        moves.push(95);
                        moves.push(93);
                    }
                    if i == 25 && field[24] == 0 && field[23] == 0 && field[22] == 0 && field[21] == 21 {
                        moves.push(25);
                        moves.push(23);
                    }
                }
            }

            // Pawn moves
            if field[i as usize] == pawn_value {
                if white {
                    if field[(i - 10) as usize] == 0 {
                        moves.push(i);
                        moves.push(i - 10);
                        if i >= 81 && i <= 88 && field[(i - 20) as usize] == 0 {
                            moves.push(i);
                            moves.push(i - 20);
                        }
                    }
                    if field[(i - 9) as usize] >= 20 {
                        moves.push(i);
                        moves.push(i - 9);
                    }
                    if field[(i - 11) as usize] >= 20 {
                        moves.push(i);
                        moves.push(i - 11);
                    }
                } else {
                    if field[(i + 10) as usize] == 0 {
                        moves.push(i);
                        moves.push(i + 10);
                        if i >= 31 && i <= 38 && field[(i + 20) as usize] == 0 {
                            moves.push(i);
                            moves.push(i + 20);
                        }
                    }
                    if field[(i + 9) as usize] < 20 && field[(i + 9) as usize] > 0 {
                        moves.push(i);
                        moves.push(i + 9);
                    }
                    if field[(i + 11) as usize] < 20 && field[(i + 11) as usize] > 0 {
                        moves.push(i);
                        moves.push(i + 11);
                    }
                }
            }

            // Knight moves
            if field[i as usize] == knight_value {
                let offsets = [-21, -19, -12, -8, 8, 12, 19, 21];
                for offset in offsets {
                    let target = i + offset;
                    if field[target as usize] == 0
                        || (field[target as usize] / 10 == if white { 2 } else { 1 } && field[target as usize] != -11)
                    {
                        moves.push(i);
                        moves.push(target);
                    }
                }
            }

            // Bishop moves
            if field[i as usize] == bishop_value {
                let offsets = [-11, -9, 9, 11];
                for offset in offsets {
                    let mut target = i + offset;
                    while field[target as usize] == 0 || field[target as usize] / 10 == if white { 2 } else { 1 } {
                        moves.push(i);
                        moves.push(target);
                        if field[target as usize] != 0 {
                            break;
                        }
                        target += offset;
                    }
                }
            }

            // Queen moves
            if field[i as usize] == queen_value {
                let offsets = [-11, -10, -9, -1, 1, 9, 10, 11];
                for offset in offsets {
                    let mut target = i + offset;
                    while (field[target as usize] == 0 || field[target as usize] / 10 == if white { 2 } else { 1 })
                        && field[target as usize] != -11
                    {
                        moves.push(i);
                        moves.push(target);
                        if field[target as usize] != 0 {
                            break;
                        }
                        target += offset;
                    }
                }
            }

            // Rook moves
            if field[i as usize] == rook_value {
                let offsets = [-10, 10, -1, 1];
                for offset in offsets {
                    let mut target = i + offset;
                    while (field[target as usize] == 0 || field[target as usize] / 10 == if white { 2 } else { 1 })
                        && field[target as usize] != -11
                    {
                        moves.push(i);
                        moves.push(target);
                        if field[target as usize] != 0 {
                            break;
                        }
                        target += offset;
                    }
                }
            }
        }
        moves
    }


    pub fn get_attack_idx_list(&self, field: &[i32], white: bool, mut target_idx: i32) -> Vec<i32> {
        let (white_king_pos, black_king_pos) = self.calc_king_positions(field);

        let mut check_idx_list = Vec::new();

        // Opponent's piece values
        let opponent_pawn = if white { 20 } else { 10 };
        let opponent_rook = if white { 21 } else { 11 };
        let opponent_knight = if white { 22 } else { 12 };
        let opponent_bishop = if white { 23 } else { 13 };
        let opponent_queen = if white { 24 } else { 14 };

        if target_idx == 0 {
            target_idx = if white { white_king_pos } else { black_king_pos };
        }

        // Pawns attacking
        if white {
            if field[(target_idx - 9) as usize] == opponent_pawn {
                check_idx_list.push(target_idx - 9);
            }
            if field[(target_idx - 11) as usize] == opponent_pawn {
                check_idx_list.push(target_idx - 11);
            }
        } else {
            if field[(target_idx + 9) as usize] == opponent_pawn {
                check_idx_list.push(target_idx + 9);
            }
            if field[(target_idx + 11) as usize] == opponent_pawn {
                check_idx_list.push(target_idx + 11);
            }
        }

        // Knights attacking
        let knight_offsets = [-12, -21, -8, -19, 12, 21, 8, 19];
        for &offset in &knight_offsets {
            if field[(target_idx + offset) as usize] == opponent_knight {
                check_idx_list.push(target_idx + offset);
            }
        }

        // Bishops and Queen attacking (Diagonals)
        let bishop_offsets = [-11, -9, 9, 11];
        for &offset in &bishop_offsets {
            let mut pos = target_idx;
            while field[(pos + offset) as usize] == 0 {
                pos += offset;
            }
            if field[(pos + offset) as usize] == opponent_bishop || field[(pos + offset) as usize] == opponent_queen {
                check_idx_list.push(pos + offset);
            }
        }

        // Rooks and Queen attacking (Horizontals and Verticals)
        let rook_offsets = [-10, -1, 1, 10];
        for &offset in &rook_offsets {
            let mut pos = target_idx;
            while field[(pos + offset) as usize] == 0 {
                pos += offset;
            }
            if field[(pos + offset) as usize] == opponent_rook || field[(pos + offset) as usize] == opponent_queen {
                check_idx_list.push(pos + offset);
            }
        }
        check_idx_list
    }

    /// Helper function to calculate the positions of the white and black kings.
    fn calc_king_positions(&self, field: &[i32]) -> (i32, i32) {
        let mut white_king_pos = -1;
        let mut black_king_pos = -1;

        for i in 21..99 {
            if field[i] == 15 {
                white_king_pos = i as i32;
            }
            if field[i] == 25 {
                black_king_pos = i as i32;
            }
        }
        (white_king_pos, black_king_pos)
    }


    /// Checks if the king is under attack.
    pub fn get_check_idx_list(&self, field: &[i32], white: bool) -> Vec<i32> {
        self.get_attack_idx_list(field, white, 0)
    }
}
