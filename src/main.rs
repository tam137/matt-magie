mod board;
mod turn;
mod log;
mod pgn;
use pgn::Pgn;
use turn::Turn;
use board::{Board, GameState};
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::env;
use std::error::Error;
use log::Log;
use std::process::Child;
use std::time::Duration;
use chrono::{Local, Datelike, Timelike};


#[derive(PartialEq)]
enum TimeControl {
    WhiteToMove,
    BlackToMove,
    AllStop,
}


fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();

    let engine_0 = args.get(1).unwrap();
    let engine_1 = args.get(2).unwrap();
    let logfile = args.get(3).unwrap();
    let pgn_path = args.get(4).unwrap();
    let event = args.get(5).unwrap();
    let site = args.get(6).unwrap();
    let round = args.get(7).unwrap();
    let time_per_game = args.get(8).unwrap();
    let log_on: bool = if args.get(9).unwrap() == ("log_on") { true } else { false };

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
        format!("{}/0", time_per_game.to_string().parse::<i32>().unwrap() / 1000),
        pgn_path.to_string(),
    );


    let mut logger = Log::new(logfile);
    logger.log("MattMagie Schachmanager 1.0 started".to_string());


    let (tx0, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx0);
    let (tx_clock, rx_clock) = mpsc::channel::<TimeControl>();

    let mut engine_process_0: Child = Command::new(engine_0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    logger.log(format!("loaded eng0 {}: {} ", engine_process_0.id(), engine_0));
    let engine_0_stdout = engine_process_0.stdout.take().ok_or("Failed to retrieve stdout")?;
    let id_engine_0: u32 = engine_process_0.id();
    send(&mut engine_process_0, "uci", &logger);    


    let mut engine_process_1: Child = Command::new(engine_1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    logger.log(format!("loaded eng1 {}: {} ", engine_process_1.id(), engine_1));
    let engine_1_stdout = engine_process_1.stdout.take().ok_or("Failed to retrieve stdout")?;
    let id_engine_1: u32 = engine_process_1.id();
    send(&mut engine_process_1, "uci", &logger);


    let _handle_0 = thread::Builder::new().name("Thread 0".to_string()).spawn(move || {
        let reader_eng0 = BufReader::new(engine_0_stdout);
        for line in reader_eng0.lines() {
            tx0.send("0_".to_string() + &line.unwrap()).unwrap();
        }
    })?;

    let _handle_1 = thread::Builder::new().name("Thread 1".to_string()).spawn(move || {
        let reader_eng1 = BufReader::new(engine_1_stdout);
        for line in reader_eng1.lines() {
            tx1.send("1_".to_string() + &line.unwrap()).unwrap();
        }
    })?;



    let time_white = Arc::new(Mutex::new(time_per_game.to_string().parse::<i32>().unwrap()));
    let time_black = Arc::new(Mutex::new(time_per_game.to_string().parse::<i32>().unwrap()));
    let time_white_clone = Arc::clone(&time_white);
    let time_black_clone = Arc::clone(&time_black);


    let _handle_1 = thread::Builder::new().name("Time_Control".to_string()).spawn(move || {

        let mut to_move = TimeControl::AllStop;
    
        loop {
            match rx_clock.try_recv() {
                Ok(message) => {
                    match message {
                        TimeControl::WhiteToMove => {
                            to_move = TimeControl::WhiteToMove;
                        },
                        TimeControl::BlackToMove => {
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
                    println!("Disconnected clock channel for TimeControl...");
                }
            }
            if to_move == TimeControl::WhiteToMove {
                *time_white_clone.lock().unwrap() -= 10;
            } else if to_move == TimeControl::BlackToMove {
                *time_black_clone.lock().unwrap() -= 10;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
    })?;



    let mut board = Board::new();
    let mut all_moves_long_algebraic = String::new();
    let mut game_status = 0;

    let mut remaining_time_white;
    let mut remaining_time_black;
    
    // mainthread loop received engine inputs from all engines
    loop {

        remaining_time_white = *time_white.lock().unwrap();
        remaining_time_black = *time_black.lock().unwrap();
    
        if remaining_time_white <= 0 {
            board.set_state(GameState::BlackWinByTime);
        } else if remaining_time_black <= 0{
            board.set_state(GameState::WhiteWinByTime);
        }

        if game_status == 2 {
            // all Engines ready for new game
            send(&mut engine_process_0, &format!("go wtime {} btime {}", remaining_time_white, remaining_time_black), &logger);
            tx_clock.send(TimeControl::WhiteToMove).unwrap();
            game_status += 1;
        }

        if check_game_over(&mut board, &tx_clock, &mut logger, &mut pgn, &all_moves_long_algebraic) {
            logger.log(format!("white_time: {} black_time: {}", remaining_time_white, remaining_time_black));
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
                    logger.log(format!("{}\t->logger\t{}", id_engine, value));
                } else {
                    logger.log(format!("{}\t->  mat\t\t{}", id_engine, value));
                }
                
        
                match msg {
                    "uciok" => {
                        send(current_engine_process, "isready", &logger);
                    }
                    "readyok" => {
                        send(current_engine_process, "ucinewgame", &logger);
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
                        let best_move: &str = &msg[9..13];
                        let mut turn: Turn = Turn::generate_turns(best_move)[0].clone();

                        Turn::enrich_promotion_move(&mut turn, &board, white);
                        let long_agrebraic = board.do_turn_and_return_long_algebraic(&turn);
                        board.add_position_for_3_move_repetition_check(board.get_fen());


                        let move_number = if board.get_pty() % 2 == 1 { format!("{}. ", board.get_pty() / 2 + 1) } else { format!("") };
                        all_moves_long_algebraic = format!("{} {}{}", all_moves_long_algebraic, move_number, long_agrebraic);
                        let _possible_turns = board.get_turn_list(!white, false); // sets GameStatus
                        
                        if _possible_turns.len() == 0 {
                            logger.log("found no moves".to_string());
                        }

                        let all_moves_str = board.get_all_made_turns()
                            .iter()
                            .map(|turn| turn.to_algebraic(false))
                            .collect::<Vec<String>>()
                            .join(" ");

                
                            if check_game_over(&mut board, &tx_clock, &mut logger, &mut pgn, &all_moves_long_algebraic) {
                                break;
                            }              

                        let all_moves = format!("position startpos moves {}", all_moves_str);
                        send(other_engine_process, &all_moves, &logger);
                        send(other_engine_process, &format!("go wtime {} btime {}", remaining_time_white, remaining_time_black), &logger);
                        if !white {
                            tx_clock.send(TimeControl::WhiteToMove).unwrap();
                        } else {
                            tx_clock.send(TimeControl::BlackToMove).unwrap();
                        }
                    }
                    _ => {}
                }
            },
            Err(_error) => {
                logger.log(board.get_fen());
                logger.log(_error.to_string());
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    send(&mut engine_process_0, "quit", &logger);
    send(&mut engine_process_1, "quit", &logger);
    logger.log("finished Matt Magie".to_string());
    std::process::exit(0);
}


fn check_game_over(board: &mut Board, tx_clock: &mpsc::Sender<TimeControl>, logger: &mut Log, pgn: &mut Pgn, all_moves_long_algebraic: &str) -> bool {
    match board.get_state() {
        &GameState::WhiteWin | &GameState::BlackWin | &GameState::WhiteWinByTime | &GameState::BlackWinByTime | &GameState::Draw => {
            tx_clock.send(TimeControl::AllStop).unwrap();
            logger.log(format!("{:?} {}", board.get_state(), board.get_fen()));
            pgn.set_moves(all_moves_long_algebraic.to_string());
            pgn.set_ply_count(format!("{}", board.get_pty()));

            let state = *board.get_state();
            let result = match state {
                GameState::WhiteWin | GameState::WhiteWinByTime => "1-0",
                GameState::BlackWin | GameState::BlackWinByTime => "0-1",
                _ => "1/2-1/2",
            };
            pgn.set_result(String::from(result));
            pgn.save();
            true
        }
        _ => false
    }
}


fn send(engine: &mut Child, command: &str, logger: &Log) {
    let command_with_newline = format!("{}\n", command);
    let stdin = engine.stdin.as_mut().expect("Failed");
    stdin.write_all(&command_with_newline.as_bytes())
        .unwrap_or_else(|err| {
            eprintln!("Failed to write to stdin Command ->: {} - {}", command, err);
        });
    stdin.flush().unwrap();
    logger.log(format!("mat\t->  {}\t{}", engine.id(), command));
}