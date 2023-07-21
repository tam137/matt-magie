
use std::fs::OpenOptions;
use chrono::Local;
use std::io::Write;


pub struct Log {
    pub(crate) path: String,
}


impl Log {

    pub fn new(path: &str) -> Log {
        Log {
            path: path.to_string(),
        }
    }

    pub fn log(&self, msg: String) {
        let timestamp = Local::now().format("%H:%M:%S");
        let log_entry = format!("{} {}", timestamp, msg + "\n");
        match OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&self.path) {
                Ok(mut file) => {
                    match file.write_all(log_entry.as_bytes()) {
                        Ok(_) => (),
                        Err(e) => println!("Error writing to file: {}", e),
                    }
                },
                Err(e) => println!("Error opening file: {}", e),
            }
    }
}