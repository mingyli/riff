use std::io;

use crate::config::Config;
use crate::diff::{Change, ChangeType};
use crate::hunk;

pub fn print_normal_hunks(config: &Config, changes: &[Change<&String>]) -> io::Result<()> {
    let hunks = hunk::get_hunks(&changes)?;
    hunks.iter().for_each(|hunk| {
        if !hunk.is_same() {
            let left_interval = hunk.left_interval();
            let left_format = if left_interval.0 == left_interval.1 {
                format!("{}", left_interval.0)
            } else {
                format!("{},{}", left_interval.0, left_interval.1)
            };
            let right_interval = hunk.right_interval();
            let right_format = if right_interval.0 == right_interval.1 {
                format!("{}", right_interval.0)
            } else {
                format!("{},{}", right_interval.0, right_interval.1)
            };

            println!("{}{}{}", left_format, hunk.change_type(), right_format);
            hunk.removals.iter().for_each(|s: &&String| {
                println!("< {}", config.color_config.removed.paint(s.as_str()));
            });
            if hunk.change_type() == ChangeType::Changed {
                println!("---");
            }
            hunk.additions.iter().for_each(|s: &&String| {
                println!("> {}", config.color_config.added.paint(s.as_str()));
            });
        }
    });
    Ok(())
}

pub fn print_unified_diffs(config: &Config, changes: &[Change<&String>]) -> io::Result<()> {
    if changes.is_empty() {
        return Ok(());
    }

    println!("--- {}", config.left_file);
    println!("+++ {}", config.right_file);

    let hunks = hunk::get_hunks(&changes)?;
    hunks.iter().for_each(|hunk| {
        println!("{}  {}", hunk.left_line, hunk.right_line);
        hunk.unchanged.iter().for_each(|s| println!(" {}", s));
        hunk.removals
            .iter()
            .for_each(|s| println!("-{}", config.color_config.removed.paint(s.as_str())));
        hunk.additions
            .iter()
            .for_each(|s| println!("+{}", config.color_config.added.paint(s.as_str())));
    });
    Ok(())
}
