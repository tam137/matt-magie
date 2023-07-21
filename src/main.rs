mod board;
mod turn;
mod log;
mod pgn;
use pgn::Pgn;
use turn::Turn;
use board::{Board, GameState};
use std::thread;
use std::sync::mpsc;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::env;
use std::error::Error;
use log::Log;
use std::process::Child;
use std::time::Duration;
use chrono::{Local, Datelike, Timelike};


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
    let time_per_move = args.get(9).unwrap();

    let now = Local::now();
    let date = format!("{:04}.{:02}.{:02}", now.year(), now.month(), now.day());
    let time = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());

    let mut pgn = Pgn::new(
        event.clone(),
        site.clone(),
        date,
        round.clone(),
        "Engine_1".to_string(),
        "Engine_2".to_string(),
        time,
        "1/0".to_string(),
        pgn_path.to_string(),
    );




    let logger = Log::new(logfile);
    logger.log("MattMagie Schachmanager beta started".to_string());

    let (tx0, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx0);

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

    let mut board = Board::new();
    let mut game_status = 0;

    // loop received engine inputs from all engines
    loop {
        
        if game_status == 2 {
            // all Engines ready for new game
            send(&mut engine_process_0, "go wtime 1000 btime 1000", &logger);
            game_status += 1;
        }

        let result = rx.recv();
        
        match result {
            Ok(value) => {
                let (id_engine, msg, current_engine_process, other_engine_process, white) = if value.starts_with("0") {
                    (id_engine_0, &value[2..], &mut engine_process_0, &mut engine_process_1, true)
                } else {
                    (id_engine_1, &value[2..], &mut engine_process_1, &mut engine_process_0, false)
                };
        
                logger.log(format!("{}\t->  mat\t\t{}", id_engine, value));
        
                match msg {
                    "uciok" => {
                        send(current_engine_process, "isready", &logger);
                    }
                    "readyok" => {
                        send(current_engine_process, "ucinewgame", &logger);
                        game_status += 1;
                    }
                    _ if msg.starts_with("bestmove") => {
                        let best_move: &str = &msg[9..13];
                        let mut turn: Turn = Turn::generate_turns(best_move)[0].clone();

                        Turn::enrich_promotion_move(&mut turn, &board, white);
                        board.do_turn(&turn);
                        let _possible_turns = board.get_turn_list(!white, false); // sets GameStatus
                        
                        if _possible_turns.len() == 0 {
                            logger.log("found no moves".to_string());
                        }

                        let all_moves_str = board.get_all_made_turns()
                            .iter()
                            .map(|turn| turn.to_algebraic())
                            .collect::<Vec<String>>()
                            .join(" ");
                
                        match board.get_state() { // if game has finished
                            &GameState::WhiteWin | &GameState::BlackWin | &GameState::Draw => {
                                logger.log(format!("{:?} {}", board.get_state(), board.get_fen()));
                                send(current_engine_process, "stop", &logger);
                                send(other_engine_process, "stop", &logger);
                                pgn.set_moves(all_moves_str);
                                pgn.save();
                                break;
                            }
                            _ => {}
                        }                

                        let all_moves = format!("position startpos moves {}", all_moves_str);
                        send(other_engine_process, &all_moves, &logger);
                        send(other_engine_process, "go wtime 1000 btime 1000", &logger);
                    }
                    _ => {}
                }
            },
            Err(_error) => {
                // received nothing ignore
                logger.log(board.get_fen());
                logger.log(_error.to_string());                
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
    logger.log("finished Matt Magie".to_string());
    Ok(())
}



fn send(engine: &mut Child, command: &str, logger: &Log) {
    let command_with_newline = format!("{}\n", command);
    let stdin = engine.stdin.as_mut().expect("Failed");
    stdin.write_all(&command_with_newline.as_bytes()).expect("Failed to write to stdin");
    stdin.flush().unwrap();
    logger.log(format!("mat\t->  {}\t{}", engine.id(), command));
}