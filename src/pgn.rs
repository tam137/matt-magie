use std::fs::OpenOptions;
use std::io::Write;

pub struct Pgn {
    pub(crate) event: String,
    pub(crate) site: String,
    pub(crate) date: String,
    pub(crate) round: String,
    pub(crate) white: String,
    pub(crate) black: String,
    pub(crate) result: String,
    pub(crate) ply_count: String,
    pub(crate) time_control: String,
    pub(crate) time: String,
    pub(crate) moves: String,
    pub(crate) path: String,
}


impl Pgn {
    pub fn new(event: String,
        site: String,
        date: String,
        round: String,
        white: String,
        black: String,
        time: String,
        time_control: String,
        path: String,
            ) -> Pgn {

        Pgn {
            event: event,
            site: site,
            date: date,
            round: round,
            white: white,
            black: black,
            time: time,
            time_control: time_control,
            result: String::new(),
            ply_count: String::new(),
            moves: String::new(),
            path: path,
        }
    }

    pub fn set_result(&mut self, result: String) {
        self.result = result;
    }

    pub fn set_ply_count(&mut self, ply_count: String) {
        self.ply_count = ply_count;
    }

    pub fn set_moves(&mut self, moves: String) {
        self.moves = moves;
    }

    pub fn save(&self) {
        let content = format!("\n\n\n[Event \"{}\"]\n[Site \"{}\"]\n[Date \"{}\"]\n[Round \"{}\"]\n[White \"{}\"]\n[Black \"{}\"]\n[Result \"{}\"]\n[TimeControl \"{}\"]\n[Time \"{}\"]\n\n{}",
        self.event,
        self.site,
        self.date,
        self.round,
        self.white,
        self.black,
        self.result,
        self.time_control,
        self.time,
        self.moves,
    );

        match OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => println!("Error writing to file: {}", e),
                    }
                },
                Err(e) => println!("Error opening file: {}", e),
            }
    }
}