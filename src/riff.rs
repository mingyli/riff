use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

use clap::{App, Arg};

mod color;
mod config;
mod diff;
mod printer;

use config::ConfigBuilder;

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
        .arg(
            Arg::with_name("normal")
                .long("normal")
                .help("Output a normal diff."),
        )
        .arg(
            Arg::with_name("unified")
                .long("unified")
                .short("u")
                .help("Output a unified diff."),
        )
        .arg(
            Arg::with_name("color")
                .long("color")
                .help("Output diff in color."),
        )
        .get_matches();

    let left_file_name = matches
        .value_of("left_file")
        .expect("This argument is required by clap.");
    let right_file_name = matches
        .value_of("right_file")
        .expect("This argument is required by clap.");
    let left_file = File::open(&left_file_name)?;
    let right_file = File::open(&right_file_name)?;

    let mut config_builder = ConfigBuilder::new()
        .with_left_file(&left_file_name)
        .with_right_file(&right_file_name);

    let left_tokens: Vec<String> = BufReader::new(left_file)
        .lines()
        .map(|line| line.unwrap())
        .collect();
    let right_tokens: Vec<String> = BufReader::new(right_file)
        .lines()
        .map(|line| line.unwrap())
        .collect();

    let changes = diff::diff(&left_tokens, &right_tokens);

    if matches.is_present("color") {
        config_builder = config_builder.with_colors();
    } else if matches.is_present("normal") {
        config_builder = config_builder.with_plain_colors();
    } else {
        config_builder = config_builder.with_colors();
    }

    let config = config_builder.build()?;

    if matches.is_present("normal") {
        printer::print_normal_hunks(&config, &changes)?;
    } else if matches.is_present("unified") {
        printer::print_unified_diffs(&config, &changes)?;
    }

    Ok(())
}
