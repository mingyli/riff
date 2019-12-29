use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

use clap::{App, Arg};

mod diff;

fn main() -> io::Result<()> {
    let matches = App::new("riff")
        .author("mingyli")
        .about("Compare files side-by-side.")
        .arg(
            Arg::with_name("left_file")
                .required(true)
                .help("The base file to compare with.")
                .value_name("FILE1"),
        )
        .arg(
            Arg::with_name("right_file")
                .required(true)
                .help("The file to compare with the base file.")
                .value_name("FILE2"),
        )
        .get_matches();

    let left_file = File::open(
        matches
            .value_of("left_file")
            .expect("This argument is required."),
    )?;
    let right_file = File::open(
        matches
            .value_of("right_file")
            .expect("This argument is required."),
    )?;

    let left_tokens: Vec<String> = BufReader::new(left_file)
        .lines()
        .map(|line| line.unwrap())
        .collect();
    let right_tokens: Vec<String> = BufReader::new(right_file)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let diffs = diff::diff(&left_tokens, &right_tokens)?;
    println!("{:?}", diffs);
    Ok(())
}
