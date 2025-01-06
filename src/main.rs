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
use model::Board;

use crate::log::log;


#[derive(PartialEq)]
enum TimeControl {
    WhiteToMove,
    BlackToMove,
    AllStop,
}


fn main() -> Result<(), Box<dyn Error>> {

    let args: Vec<String> = env::args().collect();

    let engine_0 = args.get(1).cloned().expect("MM engine_0 not defined");
    let engine_1 = args.get(2).cloned().expect("MM engine_1 not defined");
    let logfile = Arc::new(args.get(3).expect("MM logfile path not defined").to_string());
    let pgn_path = args.get(4).expect("MM pgn file path not defined").to_string();
    let event = args.get(5).expect("MM pgn event not defined").to_string();
    let site = args.get(6).expect("MM pgn site not defined").to_string();
    let round = args.get(7).expect("MM pgn round not defined").to_string();
    let time_per_game = args.get(8).expect("MM pgn time per game not defined").to_string();
    let inc_per_move_in_ms = args.get(9).expect("MM Inc per move not defined").to_string();
    let log_on: bool = if args.get(9).expect("MM log_on not defined") == ("log_on") { true } else { false };
    let debug_on: bool = if args.get(10).expect("MM log_on not defined") == ("debug_on") { true } else { false };

    log(&format!("debug is {}", debug_on), &logfile);

    let logfile_t1 = Arc::clone(&logfile);
    let logfile_t2 = Arc::clone(&logfile);
    let logfile_t3 = Arc::clone(&logfile);
    let logfile_t4 = Arc::clone(&logfile);
    
    let engine_0_t1 = engine_0.clone();
    let engine_1_t1 = engine_1.clone();
    

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


    log("Matt-Magie 1.1.3-candidate started", &logfile);

    let (tx0, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx0);
    let (tx_clock, rx_clock) = mpsc::channel::<TimeControl>();

    // load engine processes
    let mut engine_process_0 = Arc::new(Mutex::new(Command::new(engine_0)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn engine: {}", e))?));

    
    let mut engine_process_1 = Arc::new(Mutex::new(Command::new(engine_1)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn engine: {}", e))?));

    let engine_process_0_t1 = engine_process_0.clone();
    let engine_process_1_t1 = engine_process_1.clone();
    let mut engine_process_0_t2 = engine_process_0.clone();
    let mut engine_process_1_t2 = engine_process_1.clone();

    {
        match engine_process_0.lock() {
            Ok(engine_process_0_lock) => {
                log(&format!("loaded eng0 {}", engine_process_0_lock.id()), &logfile_t1);
            }
            Err(e) => {
                log(&format!("error Could not lock Engine 0, {}", e), &logfile)
            }
        };
        send(&mut engine_process_0, "uci", &logfile);
    }

    {
        match engine_process_1.lock() {
            Ok(engine_process_1_lock) => {
                log(&format!("loaded eng1 {}", engine_process_1_lock.id()), &logfile_t2);
            }
            Err(e) => {
                log(&format!("error Could not lock Engine 1, {}", e), &logfile)
            }
        };
        send(&mut engine_process_1, "uci", &logfile);
    }


    // check threads if engine processes are stil available2
    let _ = thread::Builder::new().name("Thread 0 observer".to_string()).spawn(move || {
        loop {
            match engine_process_0.lock() {
                Ok(mut engine_process_0_lock) => {
                    match engine_process_0_lock.try_wait() {
                        Ok(Some(status)) => log(&format!("Engine {} closed. Code {}", engine_0_t1, status), &logfile_t1),
                        Ok(None) => {}, // no status - engine is ok
                        Err(e) => log(&format!("error could not receive status for engine {}, {}", engine_0_t1, e), &logfile_t1),
                    }
                }
                Err(e) => {
                    log(&format!("error Could not lock Engine 1"), &logfile_t1)
                }
            };
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let _ = thread::Builder::new().name("Thread 1 observer".to_string()).spawn(move || {
        loop {
            match engine_process_1.lock() {
                Ok(mut engine_process_1_lock) => {
                    match engine_process_1_lock.try_wait() {
                        Ok(Some(status)) => log(&format!("Engine {} closed. Code {}", engine_1_t1, status), &logfile_t2),
                        Ok(None) => {}, // no status - engine is ok
                        Err(e) => log(&format!("error could not receive status for engine {}, {}", engine_1_t1, e), &logfile_t2),
                    }
                }
                Err(e) => {
                    log(&format!("error Could not lock Engine 1"), &logfile_t2)
                }
            };
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });



    // receive engine input
    let _handle_0 = thread::Builder::new().name("Thread 0".to_string()).spawn(move || {
        loop {
            match engine_process_0_t1.lock() {
                Ok(mut engine_process_0_lock) => {
                    let engine_0_stdout = engine_process_0_lock.stdout.take().expect("MM Failed to retrieve stdout Eng_0");
                    let reader_eng0 = BufReader::new(engine_0_stdout);
                    for line in reader_eng0.lines() {
                        tx0.send("0_".to_string() + &line.expect("MM read engine_0 std input failed"))
                            .expect("MM send engine_0 std input failed");
                    }
                }
                Err(e) => {
                    log(&format!("error Could not lock Engine 0"), &logfile_t3)
                }
            };
            thread::sleep(std::time::Duration::from_millis(1000));
        }
    });

    let tx1 = std::sync::mpsc::channel::<String>().0; // Simulierter Sender

    let _handle_1 = thread::Builder::new().name("Thread 1".to_string()).spawn({
        let engine_process_1_t1 = Arc::clone(&engine_process_1_t1);

        move || {
            loop {
                if let Ok(mut engine_process_1_lock) = engine_process_1_t1.lock() {
                    // Subprozess-Stdout holen
                    let engine_1_stdout = engine_process_1_lock
                        .stdout
                        .take()
                        .expect("Failed to retrieve stdout");

                    // Datei in nicht-blockierendem Modus setzen
                    let raw_fd = engine_1_stdout.as_raw_fd();
                    fcntl(raw_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).expect("Failed to set non-blocking");

                    let reader_eng1 = BufReader::new(unsafe { File::from_raw_fd(raw_fd) });

                    for line in reader_eng1.lines() {
                        match line {
                            Ok(line) => {
                                tx1.send("1_".to_string() + &line)
                                    .expect("Failed to send message");
                            }
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // Keine Daten verfügbar
                                break;
                            }
                            Err(e) => {
                                eprintln!("Error reading line: {}", e);
                                break;
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(1000));
            }
        }
    });


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
                let  mut wtime = time_white_clone.lock().expect("MM could not unlock time_white");
                if *wtime < 1000 {
                    *wtime = 1000;
                } else {
                    *wtime -= 10;
                }
            } else if to_move == TimeControl::BlackToMove {
                let  mut btime = time_black_clone.lock().expect("MM could not unlock time_white");
                if *btime < 2000 {
                    *btime = 2000;
                } else {
                    *btime -= 10;
                }
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
    })?;


    let service = Service::new();

    let mut game = UciGame::new(service.fen.set_init_board());
    let mut all_moves_long_algebraic = String::new();
    let mut game_status = 0;

    let mut remaining_time_white;
    let mut remaining_time_black;
    
    // mainthread loop received engine inputs from all engines
    loop {

        remaining_time_white = *time_white.lock().expect("MM could not unlock time_white (remaining_time)");
        remaining_time_black = *time_black.lock().expect("MM could not unlock time_white (remaining_time)");
    
        if remaining_time_white <= 0 {
            game.board.game_status = GameStatus::BlackWinByTime;
        } else if remaining_time_black <= 0{
            game.board.game_status = GameStatus::WhiteWinByTime;
        }

        if game_status == 2 {
            // all Engines ready for new game
            send(&mut engine_process_0_t2, &format!("go wtime {} btime {}", remaining_time_white, remaining_time_black), &logfile_t4);
            tx_clock.send(TimeControl::WhiteToMove).expect("MM could not send time data");
            game_status += 1;
        }

        if check_game_over(&mut game.board, &tx_clock, &logfile_t4, &mut pgn, &all_moves_long_algebraic, &service) {
            log(&format!("white_time {} winc {} black_time {} binc {}",
                remaining_time_white,
                inc_per_move_in_ms,
                remaining_time_black,
                inc_per_move_in_ms),
                &logfile_t4);
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
                log("disconnected from command queue", &logfile_t4);
                break;
            }
        };     
        
        
        match result {
            Ok(value) => {
                let (id_engine, msg, current_engine_process, other_engine_process, white) = if value.starts_with("0") {
                    ("Eng_0", &value[2..], &mut engine_process_0_t2, &mut engine_process_1_t2, true)
                } else {
                    ("Eng_1", &value[2..], &mut engine_process_1_t2, &mut engine_process_0_t2, false)
                };
        
                if msg.starts_with("log") && log_on {
                    log(&format!("{}\t->logger\t{}", id_engine, value), &logfile_t4);
                } else {
                    log(&format!("{}\t->  mat\t\t{}", id_engine, value), &logfile_t4);
                }
                
        
                match msg {
                    "uciok" => {
                        if debug_on {
                            send(current_engine_process, "debug on", &logfile_t4);
                        }
                        send(current_engine_process, "isready", &logfile_t4);
                    }
                    "readyok" => {
                        send(current_engine_process, "ucinewgame", &logfile_t4);
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
                        
                        
                        let best_move = if msg.len() > 13 {
                            &msg[9..14]
                        } else {
                            &msg[9..13]
                        };
                        
                        game.do_move(best_move);

                        let turn = NotationUtil::get_turn_from_notation(best_move);

                        let long_algebraic = if turn.promotion != 0 {
                            format!("{}{}", &msg[9..13], "=Q")
                        } else {
                            format!("{}", NotationUtil::get_long_algebraic(&msg[9..13], &game.board))
                        };
                        

                        let move_number = if game.pty % 2 == 1 { format!("{}. ", game.pty / 2 + 1) } else { format!("") };
                        all_moves_long_algebraic = format!("{} {}{}", all_moves_long_algebraic, move_number, long_algebraic);
                        let possible_turns = service.move_gen.generate_valid_moves_list(&mut game.board);
                        
                        if possible_turns.is_empty() {
                            log("found no moves", &logfile_t4);
                        }

                        let all_moves_str = game.made_moves_str.as_str();
                
                        if check_game_over(&mut game.board, &tx_clock, &logfile_t4, &mut pgn, &all_moves_long_algebraic, &service) {
                            break;
                        }              

                        let all_moves = format!("position startpos moves {}", all_moves_str);
                        send(other_engine_process, &all_moves, &logfile_t4);

                        // inc_per_move_in_ms
                        send(other_engine_process, &format!("go wtime {} winc {} btime {} binc {}",
                            remaining_time_white,
                            inc_per_move_in_ms,
                            remaining_time_black,
                            inc_per_move_in_ms
                            ),
                            &logfile_t4);

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
    //send(&mut engine_process_0, "stop", &logfile);
    //send(&mut engine_process_1, "stop", &logfile);
    send(&mut engine_process_0_t2, "quit", &logfile_t4);
    send(&mut engine_process_1_t2, "quit", &logfile_t4);
    log("finished Matt Magie", &logfile_t4);
    std::process::exit(0);
}


/// checks the Game Status. You can force a GameStatus by set the GameStatus enum in board struct
fn check_game_over(board: &mut Board,
    tx_clock: &mpsc::Sender<TimeControl>, logfile: &str, pgn: &mut Pgn, all_moves_long_algebraic: &str, service: &Service) -> bool {

    if board.move_count > 100 {
        board.game_status = GameStatus::Draw;
    }

    if board.game_status != GameStatus::Normal {
        log("Game status != Normal", logfile);
        tx_clock.send(TimeControl::AllStop).unwrap();
        log(&format!("{:?} {}", board.game_status, service.fen.get_fen(&board)), logfile);
        pgn.set_moves(all_moves_long_algebraic.to_string());
        pgn.set_ply_count(format!("{}", board.move_count));

        let state = board.game_status.clone();
        let result = match state {
            GameStatus::WhiteWin | GameStatus::WhiteWinByTime => "1-0",
            GameStatus::BlackWin | GameStatus::BlackWinByTime => "0-1",
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


/// send a cmd to the engine process
fn send(engine: &mut Arc<Mutex<Child>>, command: &str, logfile: &str) {
    let command_with_newline = format!("{}\n", command);

    match engine.lock() {
        Ok(mut engine_process) => {
            let stdin = engine_process.stdin.as_mut().expect("MM failed open std in channel");
            stdin.write_all(&command_with_newline.as_bytes())
                .unwrap_or_else(|err| {
                    eprintln!("MM Failed to write to stdin Command ->: {} - {}", command, err);
                });
            stdin.flush().unwrap();
            log(&format!("mat\t->  {}\t{}", engine_process.id(), command), logfile);
        }
        Err(e) => {
            log(&format!("error Could not lock Engine, {}", e), &logfile)
        }
    };



}