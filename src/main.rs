mod log;
mod pgn;
mod notation_util;
mod model;
mod service;
mod fen_service;
mod move_gen_service;
mod zobrist;

use notation_util::NotationUtil;
use pgn::Pgn;
use service::Service;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::env;
use std::error::Error;
use std::process::Child;
use chrono::{Local, Datelike, Timelike};
use model::UciGame;
use model::GameStatus;
use model::Turn;

use crate::log::log;


#[derive(PartialEq)]
enum TimeControl {
    WhiteToMove,
    BlackToMove,
    AllStop,
}


fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();

    let engine_0 = args.get(1).expect("MM engine_0 not defined");
    let engine_1 = args.get(2).expect("MM engine_1 not defined");
    let logfile = args.get(3).expect("MM logfile path not defined").to_string();
    let pgn_path = args.get(4).expect("MM pgn file path not defined").to_string();
    let event = args.get(5).expect("MM pgn event not defined").to_string();
    let site = args.get(6).expect("MM pgn site not defined").to_string();
    let round = args.get(7).expect("MM pgn round not defined").to_string();
    let time_per_game = args.get(8).expect("MM pgn time per game not defined").to_string();
    let inc_per_move_in_ms = args.get(9).expect("MM Inc per move not defined").to_string();
    let log_on: bool = if args.get(10).cloned().unwrap_or_default() == "log_on" { true } else { false };
    let debug_on: bool = if args.get(11).cloned().unwrap_or_default() == "debug_on" { true } else { false };
    let engine_0_options: String = args.get(12).cloned().unwrap_or_default();
    // An empty option string for engine 1 means "same as engine 0", not "no options".
    // Every caller is running an A/B match, where the two engines have to meet under identical
    // conditions; a caller that passed only argument 12 was silently leaving engine 1 on the
    // engine's own defaults -- a different hash size, which moves the transposition table hit
    // rate and with it every search rule that reads the table.
    let engine_1_options: String = match args.get(13) {
        Some(opts) if !opts.is_empty() => opts.clone(),
        _ => engine_0_options.clone(),
    };
    // Space-separated UCI moves (e.g. "e2e4 c7c5 g1f3 ..."). After an odd count Black is on move
    // at handover. Empty when no opening line was supplied.
    let opening_moves_str: String = args.get(14).cloned().unwrap_or_default();
    let opening_moves: Vec<&str> = opening_moves_str.split_whitespace().collect();

    let now = Local::now();
    let date = format!("{:04}.{:02}.{:02}", now.year(), now.month(), now.day());
    let time = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());

    let mut pgn: Pgn = Pgn::new(
        event.clone(),
        site.clone(),
        date,
        round.clone(),
        "Engine_1".to_string(),
        "Engine_2".to_string(),
        time,
        format!("{}/{}", time_per_game.to_string().parse::<i32>().expect("MM can not parse time arg") / 1000, inc_per_move_in_ms),
        "".to_string(),
        pgn_path.to_string(),
    );


    log(&format!("Matt-Magie {} started", env!("CARGO_PKG_VERSION")), &logfile);

    // Opening moves are checked like engine moves, before any engine is started: an illegal line
    // is no engine's fault, so the game is not played at all.
    let service = Service::new();
    let mut game = UciGame::new(service.fen.set_init_board());
    let mut all_moves_long_algebraic = String::new();
    let mut legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
    for mv in &opening_moves {
        if NotationUtil::find_turn_in_list(&legal_moves, mv).is_none() {
            let msg = format!("illegal opening move '{}' after '{}', game not played", mv, game.made_moves_str);
            log(&msg, &logfile);
            eprintln!("Matt-Magie: {}", msg);
            std::process::exit(1);
        }
        all_moves_long_algebraic.push_str(&apply_move_for_pgn(&mut game, mv));
        legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
    }

    let (tx0, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx0);
    let (tx_clock, rx_clock) = mpsc::channel::<TimeControl>();

    let mut engine_process_0: Child = Command::new(engine_0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    log(&format!("loaded eng0 {}: {} ", engine_process_0.id(), engine_0), &logfile);
    let engine_0_stdout = engine_process_0.stdout.take().ok_or("MM Failed to retrieve stdout")?;
    let id_engine_0: u32 = engine_process_0.id();
    send(&mut engine_process_0, "uci", &logfile);    


    let mut engine_process_1: Child = Command::new(engine_1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    log(&format!("loaded eng1 {}: {} ", engine_process_1.id(), engine_1), &logfile);
    let engine_1_stdout = engine_process_1.stdout.take().ok_or("MM Failed to retrieve stdout")?;
    let id_engine_1: u32 = engine_process_1.id();
    send(&mut engine_process_1, "uci", &logfile);


    let _handle_0 = thread::Builder::new().name("Thread 0".to_string()).spawn(move || {
        let reader_eng0 = BufReader::new(engine_0_stdout);
        for line in reader_eng0.lines() {
            tx0.send("0_".to_string() + &line.expect("MM read engine_0 std input failed")).expect("MM send engine_0 std input failed");
        }
    })?;

    let _handle_1 = thread::Builder::new().name("Thread 1".to_string()).spawn(move || {
        let reader_eng1 = BufReader::new(engine_1_stdout);
        for line in reader_eng1.lines() {
            tx1.send("1_".to_string() + &line.expect("MM read engine_1 std input failed")).expect("MM send engine_1 std input failed");
        }
    })?;


    let time_white = Arc::new(Mutex::new(time_per_game.to_string().parse::<i32>()
    .expect("MM failed to parse white time_per_game")));

    let time_black = Arc::new(Mutex::new(time_per_game.to_string().parse::<i32>()
    .expect("MM failed to parse black time_per_game")));

    let time_white_clone = Arc::clone(&time_white);
    let time_black_clone = Arc::clone(&time_black);

    let inc_per_move_in_ms = inc_per_move_in_ms.to_string().parse::<i32>().expect("MM can not parse inc per move arg");

    let _handle_1 = thread::Builder::new().name("Time_Control".to_string()).spawn(move || {

        let mut to_move = TimeControl::AllStop;
    
        loop {
            match rx_clock.try_recv() {
                Ok(message) => {
                    match message {
                        TimeControl::WhiteToMove => {
                            *time_white_clone.lock().expect("MM could not unlock time_white") += inc_per_move_in_ms;
                            to_move = TimeControl::WhiteToMove;
                        },
                        TimeControl::BlackToMove => {
                            *time_black_clone.lock().expect("MM could not unlock time_white") += inc_per_move_in_ms;
                            to_move = TimeControl::BlackToMove;
                        },
                        TimeControl::AllStop => {
                            to_move = TimeControl::AllStop;
                        },

                    }
                },
                Err(mpsc::TryRecvError::Empty) => {
                    // do nothing proceed...
                },
                Err(mpsc::TryRecvError::Disconnected) => {
                    panic!("Disconnected clock channel for TimeControl...");
                }
            }
            if to_move == TimeControl::WhiteToMove {
                let mut wtime = time_white_clone.lock().expect("MM could not unlock time_white");
                *wtime -= 10;
            } else if to_move == TimeControl::BlackToMove {
                let mut btime = time_black_clone.lock().expect("MM could not unlock time_white");
                *btime -= 10;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
    })?;


    let mut game_status = 0;

    let mut remaining_time_white;
    let mut remaining_time_black;
    
    // mainthread loop received engine inputs from all engines
    loop {

        // Check if either engine process has exited unexpectedly
        if let Ok(Some(status)) = engine_process_0.try_wait() {
            log(&format!("Engine 0 exited unexpectedly: {:?}", status), &logfile);
            game.board.game_status = GameStatus::BlackWinByTime;
        } else if let Ok(Some(status)) = engine_process_1.try_wait() {
            log(&format!("Engine 1 exited unexpectedly: {:?}", status), &logfile);
            game.board.game_status = GameStatus::WhiteWinByTime;
        }

        remaining_time_white = *time_white.lock().expect("MM could not unlock time_white (remaining_time)");
        remaining_time_black = *time_black.lock().expect("MM could not unlock time_white (remaining_time)");
    
        if remaining_time_white <= 0 {
            game.board.game_status = GameStatus::BlackWinByTime;
        } else if remaining_time_black <= 0 {
            game.board.game_status = GameStatus::WhiteWinByTime;
        }

        if game_status == 2 {
            // all Engines ready for new game
            let go_cmd = format!("go wtime {} winc {} btime {} binc {}",
                remaining_time_white, inc_per_move_in_ms, remaining_time_black, inc_per_move_in_ms);
            if game.board.white_to_move {
                send(&mut engine_process_0, &go_cmd, &logfile);
                tx_clock.send(TimeControl::WhiteToMove).expect("MM could not send time data");
            } else {
                send(&mut engine_process_1, &go_cmd, &logfile);
                tx_clock.send(TimeControl::BlackToMove).expect("MM could not send time data");
            }
            game_status += 1;
        }

        if check_game_over(&mut game, &tx_clock, &logfile, &mut pgn, &all_moves_long_algebraic, &service) {
            log(&format!("white_time {} winc {} black_time {} binc {}",
                remaining_time_white,
                inc_per_move_in_ms,
                remaining_time_black,
                inc_per_move_in_ms),
                &logfile);
            break;
        } 

        let result: Result<String, mpsc::RecvError>;

        result = match rx.try_recv() {
            Ok(message) => Ok(message),
            Err(mpsc::TryRecvError::Empty) => {
                thread::sleep(std::time::Duration::from_millis(5));
                continue;
            },
            Err(mpsc::TryRecvError::Disconnected) => {
                log("disconnected from command queue", &logfile);
                break;
            }
        };     
        
        
        match result {
            Ok(value) => {
                let (id_engine, msg, current_engine_process, other_engine_process, white) = if value.starts_with("0") {
                    (id_engine_0, &value[2..], &mut engine_process_0, &mut engine_process_1, true)
                } else {
                    (id_engine_1, &value[2..], &mut engine_process_1, &mut engine_process_0, false)
                };
        
                if msg.starts_with("log") && log_on {
                    log(&format!("{}\t->logger\t{}", id_engine, value), &logfile);
                } else {
                    log(&format!("{}\t->  mat\t\t{}", id_engine, value), &logfile);
                }
                
        
                match msg {
                    "uciok" => {
                        let opts = if white { &engine_0_options } else { &engine_1_options };
                        if !opts.is_empty() {
                            for opt in opts.split(',') {
                                if let Some((name, val)) = parse_option(opt) {
                                    send(current_engine_process, &format!("setoption name {} value {}", name, val), &logfile);
                                }
                            }
                        }
                        if debug_on {
                            send(current_engine_process, "debug on", &logfile);
                        }
                        send(current_engine_process, "isready", &logfile);
                    }
                    "readyok" => {
                        send(current_engine_process, "ucinewgame", &logfile);
                        if white == game.board.white_to_move && !opening_moves_str.is_empty() {
                            send(current_engine_process, &format!("position startpos moves {}", opening_moves_str), &logfile);
                        }
                        game_status += 1;
                    }
                    _ if msg.starts_with("id name") => {
                        if white {
                            pgn.set_white_name(&msg[8..]);
                        } else {
                            pgn.set_black_name(&msg[8..]);
                        }    
                    }
                    _ if msg.starts_with("bestmove") => {

                        match apply_engine_move(&mut game, &legal_moves, msg, white) {
                            Ok(pgn_fragment) => all_moves_long_algebraic.push_str(&pgn_fragment),
                            Err(reason) => {
                                let legal: Vec<String> = legal_moves.iter().map(|turn| turn.to_algebraic()).collect();
                                log(&format!("illegal move by {} ({}): {}; fen {}; legal moves: {}",
                                    id_engine,
                                    if white { "white" } else { "black" },
                                    reason,
                                    service.fen.get_fen(&game.board),
                                    legal.join(" ")),
                                    &logfile);
                                game.board.game_status = if white { GameStatus::BlackWinByIllegalMove } else { GameStatus::WhiteWinByIllegalMove };
                                check_game_over(&mut game, &tx_clock, &logfile, &mut pgn, &all_moves_long_algebraic, &service);
                                break;
                            }
                        }

                        legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);

                        if legal_moves.is_empty() {
                            log("found no moves", &logfile);
                        }

                        if check_game_over(&mut game, &tx_clock, &logfile, &mut pgn, &all_moves_long_algebraic, &service) {
                            break;
                        }              

                        let all_moves = format!("position startpos moves {}", game.made_moves_str);
                        send(other_engine_process, &all_moves, &logfile);

                        // inc_per_move_in_ms
                        send(other_engine_process, &format!("go wtime {} winc {} btime {} binc {}",
                            remaining_time_white,
                            inc_per_move_in_ms,
                            remaining_time_black,
                            inc_per_move_in_ms
                            ),
                            &logfile);

                        if !white {
                            tx_clock.send(TimeControl::WhiteToMove).expect("MM could not send white time command");
                        } else {
                            tx_clock.send(TimeControl::BlackToMove).expect("MM could not send black time command");
                        }
                    }
                    _ => {}
                }
            },
            Err(_error) => {
                panic!("cannot be here, message is OK");
            }
        }
    }
    send(&mut engine_process_0, "stop", &logfile);
    send(&mut engine_process_1, "stop", &logfile);
    send(&mut engine_process_0, "quit", &logfile);
    send(&mut engine_process_1, "quit", &logfile);
    log("finished Matt Magie", &logfile);
    std::process::exit(0);
}

// Applies one legal move (UCI notation) to `game` and returns its PGN fragment. Opening and
// engine moves both go through here, so they share one continuous, correctly-numbered move string.
fn apply_move_for_pgn(game: &mut UciGame, uci_move: &str) -> String {
    game.do_move(uci_move);
    let long_algebraic = NotationUtil::get_long_algebraic(uci_move, &game.board);
    let move_number = if game.pty % 2 == 1 { format!("{}. ", game.pty / 2 + 1) } else { format!("") };
    format!(" {}{}", move_number, long_algebraic)
}

// Checks an engine's `bestmove` line against the legal moves of the side to move and plays it.
// Returns the move's PGN fragment, or why the move was rejected - the sender then loses the game.
fn apply_engine_move(game: &mut UciGame, legal_moves: &[Turn], msg: &str, sender_white: bool) -> Result<String, String> {
    // "bestmove e2e4", possibly followed by "ponder e7e5"
    let uci_move = msg.split_whitespace().nth(1).ok_or_else(|| "no move after bestmove".to_string())?;
    if sender_white != game.board.white_to_move {
        return Err(format!("'{}' sent while the other side is to move", uci_move));
    }
    if NotationUtil::try_turn_from_notation(uci_move).is_none() {
        return Err(format!("'{}' is not a readable UCI move", uci_move));
    }
    if NotationUtil::find_turn_in_list(legal_moves, uci_move).is_none() {
        return Err(format!("'{}' is not a legal move", uci_move));
    }
    Ok(apply_move_for_pgn(game, uci_move))
}

fn parse_option(opt: &str) -> Option<(String, String)> {
    let opt = opt.trim();
    if opt.is_empty() {
        return None;
    }
    let parts: Vec<&str> = if opt.contains('=') {
        opt.splitn(2, '=').collect()
    } else if opt.contains(':') {
        opt.splitn(2, ':').collect()
    } else {
        return None;
    };

    if parts.len() == 2 {
        let name = parts[0].trim().to_string();
        let value = parts[1].trim().to_string();
        if !name.is_empty() && !value.is_empty() {
            return Some((name, value));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // Plays `moves` through the same check as engine moves, the legal moves regenerated after each.
    fn play(service: &Service, game: &mut UciGame, moves: &str) {
        for uci_move in moves.split_whitespace() {
            let legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
            let white = game.board.white_to_move;
            let result = apply_engine_move(game, &legal_moves, &format!("bestmove {}", uci_move), white);
            assert!(result.is_ok(), "{}: {:?}", uci_move, result);
        }
        service.move_gen.generate_valid_moves_list(&mut game.board);
    }

    use crate::model::MOVE_LIMIT;

    fn game_from(service: &Service, fen: &str) -> UciGame {
        UciGame::new(service.fen.set_fen(fen))
    }

    #[test]
    fn test_halfmove_clock_counts_quiet_moves_and_resets_on_pawn_moves_and_captures() {
        let service = Service::new();
        let mut game = UciGame::new(service.fen.set_init_board());
        play(&service, &mut game, "g1f3 g8f6 b1c3");
        assert_eq!(game.halfmove_clock, 3);
        play(&service, &mut game, "e7e5");                 // black pawn move
        assert_eq!(game.halfmove_clock, 0);
        play(&service, &mut game, "f3e5");                 // capture
        assert_eq!(game.halfmove_clock, 0);
        play(&service, &mut game, "f8c5 e2e3");            // white pawn move
        assert_eq!(game.halfmove_clock, 0);
        play(&service, &mut game, "e8g8 f1e2 b8c6 e1g1");  // castling is quiet
        assert_eq!(game.halfmove_clock, 4);
        // en passant
        let mut game = game_from(&service, "4k3/8/8/8/3p4/8/4P3/4K3 w - - 0 1");
        play(&service, &mut game, "e1d1 e8d8 d1e1 d8e8");
        assert_eq!(game.halfmove_clock, 4);
        play(&service, &mut game, "e2e4 d4e3");
        assert_eq!(game.halfmove_clock, 0);
    }

    #[test]
    fn test_fifty_move_rule_draws_at_a_hundred_quiet_plies() {
        let service = Service::new();
        let mut game = game_from(&service, "4k3/8/8/8/8/8/8/R3K3 w - - 0 1");
        game.halfmove_clock = 98;
        play(&service, &mut game, "a1a2");
        game.apply_draw_rules();
        assert_eq!(game.halfmove_clock, 99);
        assert_eq!(game.board.game_status, GameStatus::Normal);
        play(&service, &mut game, "e8d8");
        game.apply_draw_rules();
        assert_eq!(game.board.game_status, GameStatus::DrawByFiftyMoves);
    }

    #[test]
    fn test_mate_on_the_hundredth_ply_is_a_win() {
        let service = Service::new();
        let mut game = game_from(&service, "6k1/5ppp/8/8/8/8/8/R5K1 w - - 0 1");
        game.halfmove_clock = 99;
        play(&service, &mut game, "a1a8");
        game.apply_draw_rules();
        assert_eq!(game.halfmove_clock, 100);
        assert_eq!(game.board.game_status, GameStatus::WhiteWin);
    }

    #[test]
    fn test_insufficient_material_draws_only_dead_positions() {
        let service = Service::new();
        for (fen, dead) in [
            ("8/8/8/4k3/8/8/8/4K3 w - - 0 1", true),        // K v K
            ("8/8/8/4k3/8/8/8/2B1K3 w - - 0 1", true),      // KB v K
            ("8/8/8/4k3/8/8/8/1N2K3 b - - 0 1", true),      // KN v K
            ("8/8/8/4k3/8/2b5/8/2B1K3 w - - 0 1", true),    // bishops c3 and c1, both dark
            ("8/8/8/4k3/1b6/8/8/2B1K3 w - - 0 1", true),    // b4 and c1, both dark
            ("8/8/8/4k3/8/3b4/8/2B1K3 w - - 0 1", false),   // d3 light, c1 dark
            ("8/8/8/4k3/8/3n4/8/2B1K3 w - - 0 1", false),   // KB v KN
            ("8/8/8/4k3/8/3n4/8/1N2K3 w - - 0 1", false),   // KN v KN
            ("8/8/8/4k3/8/8/8/1NN1K3 w - - 0 1", false),    // KNN v K
            ("8/8/8/4k3/8/8/4P3/4K3 w - - 0 1", false),     // KP v K
            ("8/8/8/4k3/8/8/8/R3K3 w - - 0 1", false),      // KR v K
            ("8/8/8/4k3/8/8/8/Q3K3 w - - 0 1", false),      // KQ v K
        ] {
            let mut game = game_from(&service, fen);
            game.apply_draw_rules();
            let expected = if dead { GameStatus::DrawByInsufficientMaterial } else { GameStatus::Normal };
            assert_eq!(game.board.game_status, expected, "{}", fen);
        }
        // a capture that leaves K v K
        let mut game = game_from(&service, "8/8/8/8/3k4/8/3r4/3K4 w - - 0 1");
        play(&service, &mut game, "d1d2");
        game.apply_draw_rules();
        assert_eq!(game.board.game_status, GameStatus::DrawByInsufficientMaterial);
    }

    #[test]
    fn test_move_limit_draws_only_past_move_limit() {
        let service = Service::new();
        let mut game = UciGame::new(service.fen.set_init_board());
        game.board.move_count = MOVE_LIMIT;
        game.apply_draw_rules();
        assert_eq!(game.board.game_status, GameStatus::Normal);
        game.board.move_count = MOVE_LIMIT + 1;
        game.apply_draw_rules();
        assert_eq!(game.board.game_status, GameStatus::DrawByMoveLimit);
        // move 101 is no longer the end
        let mut game = UciGame::new(service.fen.set_init_board());
        game.board.move_count = 101;
        game.apply_draw_rules();
        assert_eq!(game.board.game_status, GameStatus::Normal);
    }

    #[test]
    fn test_pgn_writes_ply_count_and_termination() {
        let path = std::env::temp_dir().join(format!("mm_test_{}.pgn", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut pgn = Pgn::new(String::new(), String::new(), String::new(), String::new(), String::new(),
            String::new(), String::new(), String::new(), String::new(), path.to_string_lossy().to_string());
        pgn.set_termination(&format!("{:?}", GameStatus::DrawByFiftyMoves));
        pgn.set_ply_count("161".to_string());
        pgn.set_result("1/2-1/2".to_string());
        pgn.save();
        let text = std::fs::read_to_string(&path).unwrap();
        let _ = std::fs::remove_file(&path);
        assert!(text.contains("[Termination \"DrawByFiftyMoves\"]\n[PlyCount \"161\"]\n"), "{}", text);
    }

    #[test]
    fn test_parse_option() {
        assert_eq!(parse_option("Hash=128"), Some(("Hash".to_string(), "128".to_string())));
        assert_eq!(parse_option("Threads:4"), Some(("Threads".to_string(), "4".to_string())));
        assert_eq!(parse_option("  Ponder  =  true  "), Some(("Ponder".to_string(), "true".to_string())));
        assert_eq!(parse_option("InvalidOption"), None);
        assert_eq!(parse_option("="), None);
        assert_eq!(parse_option(""), None);
    }

    #[test]
    fn test_underpromotion_puts_the_chosen_piece_on_the_board_and_in_the_pgn() {
        let service = Service::new();
        // (side to move, move, target square, piece expected there)
        for (side, uci_move, target, piece) in [
            ("w", "b7b8q", 22, 14), ("w", "b7b8r", 22, 11), ("w", "b7b8b", 22, 13), ("w", "b7b8n", 22, 12),
            ("w", "b7a8r", 21, 11),
            ("b", "b2b1q", 92, 24), ("b", "b2b1r", 92, 21), ("b", "b2b1b", 92, 23), ("b", "b2b1n", 92, 22),
            ("b", "b2a1b", 91, 23),
        ] {
            let mut game = UciGame::new(service.fen.set_fen(&format!("r6k/1P6/8/8/8/8/1p6/R6K {} - - 0 1", side)));
            let fragment = apply_move_for_pgn(&mut game, uci_move);
            assert_eq!(game.board.field[target], piece, "{}", uci_move);
            let letter = uci_move[4..5].to_uppercase();
            assert_eq!(fragment, format!(" 1. {}={}", &uci_move[0..4], letter), "{}", uci_move);
        }
    }

    #[test]
    fn test_pgn_writes_the_promotion_piece() {
        let mut pgn = Pgn::new(String::new(), String::new(), String::new(), String::new(), String::new(),
            String::new(), String::new(), String::new(), String::new(), String::new());
        pgn.set_moves(" 1. b7b8=R b2a1=N".to_string());
        assert_eq!(pgn.moves, "1. b8=R bxa1=N");
    }

    // Round robin 2026-10-12, game started 18:46:39: SleepMind (White) against suprah-0.46.2.
    // SleepMind played h7h8r; Matt-Magie 1.4.5 made it a queen, so its board drifted from the
    // engines' and, after b5a4, that queen "captured" the king -> panic, no PGN.
    const RR_2026_10_12_CRASH_GAME: &str = "c2c4 b7b6 b1c3 e7e6 d2d4 c8b7 d1c2 f8b4 e2e4 d8h4 \
        f1d3 b4c3 b2c3 f7f5 g2g3 h4h5 f2f3 g8e7 e4f5 e6f5 c1f4 d7d6 c2e2 e8g8 e2e6 f8f7 e1c1 g7g5 \
        f4d2 g5g4 g1e2 g4f3 e2f4 h5g4 h2h3 g4g5 f4d5 b7d5 e6f7 g8f7 d2g5 d5e4 g5e7 f7e7 d3e4 f5e4 \
        h1e1 d6d5 c4d5 e7d6 e1e4 d6d5 e4e8 c7c5 d1f1 c5d4 f1f3 d5c4 c3d4 c4d4 f3f8 a7a5 e8b8 a8a7 \
        b8b6 a5a4 c1d2 d4e5 d2e3 a7c7 f8e8 e5d5 b6a6 c7c3 e3f4 c3c4 f4f3 c4b4 a6a5 d5d6 e8e4 b4b7 \
        a5a4 b7f7 e4f4 f7b7 h3h4 d6c5 a4a6 b7b1 a6h6 b1f1 f3g4 f1d1 h6h7 c5b6 h4h5 b6a5 g4g5 a5b6 \
        g3g4 d1d5 f4f5 d5d8 f5f6 b6c5 h5h6 d8g8 h7g7 g8a8 h6h7 c5b4 g7g8 a8a2 h7h8r b4b3 g8c8 a2a5 \
        g5g6 a5b5 h8h3 b3b4 f6f4 b4a5 f4f5 b5f5 g6f5 a5b4 g4g5 b4b5 h3b3 b5a4";

    #[test]
    fn test_round_robin_underpromotion_game_replays() {
        let service = Service::new();
        let mut game = UciGame::new(service.fen.set_init_board());
        let mut legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
        let moves: Vec<&str> = RR_2026_10_12_CRASH_GAME.split_whitespace().collect();
        assert_eq!(moves.len(), 132);
        for uci_move in moves {
            let white = game.board.white_to_move;
            let result = apply_engine_move(&mut game, &legal_moves, &format!("bestmove {}", uci_move), white);
            assert!(result.is_ok(), "{} after {}: {:?}", uci_move, game.made_moves_str, result);
            if uci_move == "h7h8r" {
                assert_eq!(game.board.field[28], 11, "h8 must hold a white rook");
            }
            legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
        }
        assert_eq!(game.board.game_status, GameStatus::Normal);
        assert!(!legal_moves.is_empty());
    }

    #[test]
    fn test_engine_move_is_checked_against_the_legal_moves() {
        let service = Service::new();
        // Start position, White to move. (message, sent by White, accepted)
        for (msg, white, accepted) in [
            ("bestmove e2e4", true, true),
            ("bestmove g1f3 ponder g8f6", true, true),
            ("bestmove e2e5", true, false),     // illegal
            ("bestmove xyz", true, false),      // unreadable
            ("bestmove 0000", true, false),
            ("bestmove (none)", true, false),
            ("bestmove", true, false),          // no move at all
            ("bestmove E2E4", true, false),     // UCI moves are lower case
            ("bestmove e2e4q", true, false),    // promotion letter on a move to the 4th rank
            ("bestmove e7e5", false, false),    // Black moves while White is to move
            ("bestmove e2e4", false, false),    // White's move, sent by Black
        ] {
            let mut game = UciGame::new(service.fen.set_init_board());
            let legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
            let result = apply_engine_move(&mut game, &legal_moves, msg, white);
            assert_eq!(result.is_ok(), accepted, "'{}' from {}: {:?}", msg, if white { "White" } else { "Black" }, result);
            if !accepted {
                assert_eq!(game.pty, 0, "a rejected move must leave the game untouched: {}", msg);
                assert!(game.made_moves_str.is_empty(), "{}", msg);
            }
        }

        let mut game = UciGame::new(service.fen.set_init_board());
        let legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
        assert_eq!(apply_engine_move(&mut game, &legal_moves, "bestmove g1f3 ponder g8f6", true), Ok(" 1. Ng1f3".to_string()));
        assert_eq!(game.made_moves_str, "g1f3");
    }

    #[test]
    fn test_promotion_needs_its_piece_letter() {
        let service = Service::new();
        for (side, msg, accepted) in [
            ("w", "bestmove b7b8", false),
            ("w", "bestmove b7b8k", false),
            ("w", "bestmove b7b8r", true),
            ("b", "bestmove b2a1", false),
            ("b", "bestmove b2a1b", true),
        ] {
            let mut game = UciGame::new(service.fen.set_fen(&format!("r6k/1P6/8/8/8/8/1p6/R6K {} - - 0 1", side)));
            let legal_moves = service.move_gen.generate_valid_moves_list(&mut game.board);
            let result = apply_engine_move(&mut game, &legal_moves, msg, side == "w");
            assert_eq!(result.is_ok(), accepted, "{}: {:?}", msg, result);
        }
    }
}


fn check_game_over(game: &mut UciGame,
    tx_clock: &mpsc::Sender<TimeControl>, logfile: &str, pgn: &mut Pgn, all_moves_long_algebraic: &str, service: &Service) -> bool {

    game.apply_draw_rules();
    let board = &game.board;

    if board.game_status != GameStatus::Normal {
        log("Game status != Normal", logfile);
        tx_clock.send(TimeControl::AllStop).unwrap();
        log(&format!("{:?} {}", board.game_status, service.fen.get_fen(&board)), logfile);
        pgn.set_moves(all_moves_long_algebraic.to_string());
        pgn.set_ply_count(format!("{}", game.pty));

        let state = board.game_status.clone();
        let result = match state {
            GameStatus::WhiteWin | GameStatus::WhiteWinByTime | GameStatus::WhiteWinByIllegalMove => "1-0",
            GameStatus::BlackWin | GameStatus::BlackWinByTime | GameStatus::BlackWinByIllegalMove => "0-1",
            _ => "1/2-1/2",
        };
        pgn.set_termination(&format!("{:?}", state));
        pgn.set_result(String::from(result));
        pgn.save();
        true
    } else {
        false
    }
}



fn send(engine: &mut Child, command: &str, logfile: &str) {
    let command_with_newline = format!("{}\n", command);
    let stdin = engine.stdin.as_mut().expect("Failed");
    stdin.write_all(&command_with_newline.as_bytes())
        .unwrap_or_else(|err| {
            eprintln!("Failed to write to stdin Command ->: {} - {}", command, err);
        });
    stdin.flush().unwrap();
    log(&format!("mat\t->  {}\t{}", engine.id(), command), logfile);
}