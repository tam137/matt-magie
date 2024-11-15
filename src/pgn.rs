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
        let modified_moves = moves
            .split_whitespace()
            .map(|mv| {
                if mv.contains('=') {
                    // Es handelt sich um eine Bauernumwandlung
                    let parts: Vec<&str> = mv.split('=').collect();
                    if parts.len() != 2 {
                        // Ungültiges Format, Rückgabe des Originalzugs
                        return mv.to_string();
                    }
                    let promotion_piece = parts[1]; // Z.B. "Q"
                    let move_without_promotion = parts[0]; // Z.B. "e7e8" oder "d7e8"
    
                    // Extrahiere das Von- und Zielfeld
                    let from_square = &move_without_promotion[0..2]; // Z.B. "e7"
                    let to_square = &move_without_promotion[2..4];   // Z.B. "e8"
    
                    let from_file = &from_square[0..1]; // Z.B. "e"
                    let to_file = &to_square[0..1];     // Z.B. "e" oder "d"
    
                    if from_file != to_file {
                        // Schlagzug bei der Umwandlung
                        format!("{}x{}={}", from_file, to_square, promotion_piece)
                    } else {
                        // Keine Schlagzug, normale Umwandlung
                        format!("{}={}", to_square, promotion_piece)
                    }
                } else {
                    // Behandlung von Rochade und anderen Zügen
                    match mv {
                        "Ke1g1" | "Ke8g8" => "0-0".to_string(),
                        "Ke1c1" | "Ke8c8" => "0-0-0".to_string(),
                        _ => mv.to_string(),
                    }
                }
            })
            .collect::<Vec<String>>()
            .join(" ");
        self.moves = modified_moves;
    }
    
    

    pub fn set_white_name(&mut self, name: &str) {
        self.white = String::from(name);
    }

    pub fn set_black_name(&mut self, name: &str) {
        self.black = String::from(name);
    }

    pub fn save(&self) {
        let content = format!("[Event \"{}\"]\n[Site \"{}\"]\n[Date \"{}\"]\n[Round \"{}\"]\n[White \"{}\"]\n[Black \"{}\"]\n[Result \"{}\"]\n[TimeControl \"{}\"]\n[Time \"{}\"]\n{} {}\n\n",
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
        self.result,
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