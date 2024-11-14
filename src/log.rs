
use std::fs::OpenOptions;
use chrono::Local;
use std::io::Write;

pub fn log(msg: &str, path: &str) {
    let timestamp = Local::now().format("%H:%M:%S%.3f");
    let log_entry = format!("{} {}", timestamp, msg.to_string() + "\n");
    match OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(path) {
            Ok(mut file) => {
                match file.write_all(log_entry.as_bytes()) {
                    Ok(_) => (),
                    Err(e) => println!("Error writing to file: {}", e),
                }
            },
            Err(e) => println!("Error opening file: {}", e),
        }
}
