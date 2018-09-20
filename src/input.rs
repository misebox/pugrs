use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;

pub fn read_file(filename: &str) -> String {
    // file open
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<io::Result<Vec<String>>>().unwrap();
    lines.join("\n") // CRLF => LF
}
