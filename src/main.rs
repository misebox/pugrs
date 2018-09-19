extern crate regex;
use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
//use std::fs::OpenOptions;

fn main() {
    // regex
    let re = Regex::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    let text = "2012-03-14, 2013-01-01 and 2014-07-05";
    for cap in re.captures_iter(text) {
        println!("Month: {} Day: {} Year: {}", &cap[2], &cap[3], &cap[1]);
    }

    // args
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() <= 0 {
        return;
    }
    let filename: &str = &args[0];

    // file open
    let mut file = File::open(filename).unwrap();
    let mut reader = BufReader::new(file);
    let mut lines: Vec<String> = reader.lines().collect::<io::Result<Vec<String>>>().unwrap();
    for l in lines {
        println!("{}", l);
    }
}
