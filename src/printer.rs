use std::io;

use itertools::{Either, Itertools};

use crate::config::Config;
use crate::diff::{Change, ChangeType};

pub fn print_normal_hunks(config: &Config, changes: &[Change<&String>]) -> io::Result<()> {
    let hunks = get_hunks(&changes)?;
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

    let hunks = get_hunks(&changes)?;
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

/// A hunk is a contiguous subsequence of tokens from two sequences.
/// It is either a sequence of identical tokens from both sequences
/// or a combination of removals from the left sequence and additions
/// to the right sequence. An invariant is that either `unchanged` is empty
/// or `additions` and `removals` are empty.
#[derive(Debug, PartialEq)]
struct Hunk<'a, T> {
    left_line: usize,
    right_line: usize,
    additions: Vec<&'a T>,
    removals: Vec<&'a T>,
    unchanged: Vec<&'a T>,
}

impl<'a, T> Hunk<'a, T> {
    fn is_same(&self) -> bool {
        !self.unchanged.is_empty()
    }

    fn is_addition(&self) -> bool {
        !self.additions.is_empty()
    }

    fn is_removal(&self) -> bool {
        !self.removals.is_empty()
    }

    fn is_change(&self) -> bool {
        self.is_addition() && self.is_removal()
    }

    fn change_type(&self) -> ChangeType {
        if self.is_change() {
            ChangeType::Changed
        } else if self.is_addition() {
            ChangeType::Added
        } else {
            ChangeType::Deleted
        }
    }

    fn left_interval(&self) -> (usize, usize) {
        if self.removals.is_empty() {
            (self.left_line, self.left_line)
        } else {
            (self.left_line, self.left_line + self.removals.len() - 1)
        }
    }

    fn right_interval(&self) -> (usize, usize) {
        if self.additions.is_empty() {
            (self.right_line, self.right_line)
        } else {
            (self.right_line, self.right_line + self.additions.len() - 1)
        }
    }
}

fn get_hunks<'a, T>(changes: &[Change<&'a T>]) -> io::Result<Vec<Hunk<'a, T>>> {
    let groups = changes.iter().group_by(|change| match change {
        Change::Same(_) => true,
        _ => false,
    });
    let (_, _, hunks) = groups.into_iter().fold(
        Ok((0, 0, Vec::new())),
        |result: io::Result<(usize, usize, Vec<Hunk<T>>)>, (is_same, group)| {
            let (mut left_line, mut right_line, mut hunks) = result?;
            let hunk = if is_same {
                let unchanged: Vec<&T> = group.map(Change::get).cloned().collect();
                Hunk {
                    left_line: left_line + !unchanged.is_empty() as usize,
                    right_line: right_line + !unchanged.is_empty() as usize,
                    additions: vec![],
                    removals: vec![],
                    unchanged,
                }
            } else {
                let eithers: io::Result<Vec<Either<&&T, &&T>>> = group
                    .map(|change| match change {
                        Change::Removed(s) => Ok(Either::Left(s)),
                        Change::Added(s) => Ok(Either::Right(s)),
                        Change::Same(_) => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "Change::Same should be filtered out.",
                        )),
                    })
                    .collect();
                let (removals, additions): (Vec<&T>, Vec<&T>) =
                    eithers?.iter().partition_map(|&either| either);
                Hunk {
                    left_line: left_line + !removals.is_empty() as usize,
                    right_line: right_line + !additions.is_empty() as usize,
                    additions,
                    removals,
                    unchanged: vec![],
                }
            };
            left_line += hunk.unchanged.len() + hunk.removals.len();
            right_line += hunk.unchanged.len() + hunk.additions.len();
            hunks.push(hunk);
            Ok((left_line, right_line, hunks))
        },
    )?;
    Ok(hunks)
}

#[test]
fn test_get_hunks() -> io::Result<()> {
    let changes = vec![
        Change::Added(&1),
        Change::Same(&2),
        Change::Same(&3),
        Change::Removed(&4),
    ];

    let hunks: Vec<Hunk<i32>> = get_hunks(&changes)?;
    assert_eq!(
        hunks,
        vec![
            Hunk {
                left_line: 0,
                right_line: 1,
                additions: vec![&1],
                removals: vec![],
                unchanged: vec![],
            },
            Hunk {
                left_line: 1,
                right_line: 2,
                additions: vec![],
                removals: vec![],
                unchanged: vec![&2, &3],
            },
            Hunk {
                left_line: 3,
                right_line: 3,
                additions: vec![],
                removals: vec![&4],
                unchanged: vec![],
            },
        ]
    );
    Ok(())
}
