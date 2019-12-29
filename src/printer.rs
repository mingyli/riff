use itertools::{Either, Itertools};

use crate::config::Config;
use crate::Change;

pub fn print_normal_hunks(config: &Config, changes: &[Change<&String>]) {
    let groups = changes.iter().group_by(|change| match change {
        Change::Same(_) => true,
        _ => false,
    });

    let mut left_line = 0;
    let mut right_line = 0;

    for (same, group) in &groups {
        if same {
            let size = group.count();
            left_line += size;
            right_line += size;
        } else {
            let (removals, additions): (Vec<&String>, Vec<&String>) =
                group.partition_map(|change| match change {
                    Change::Removed(s) => Either::Left(s),
                    Change::Added(s) => Either::Right(s),
                    Change::Same(_) => panic!("Change::Same should already be filtered out."),
                });

            let change_type = if !removals.is_empty() && !additions.is_empty() {
                'c'
            } else if removals.is_empty() {
                'a'
            } else {
                'd'
            };

            let left_interval = if removals.is_empty() {
                (left_line, left_line)
            } else {
                (left_line + 1, left_line + removals.len())
            };
            let left_format = if left_interval.0 == left_interval.1 {
                format!("{}", left_interval.0)
            } else {
                format!("{},{}", left_interval.0, left_interval.1)
            };
            let right_interval = if additions.is_empty() {
                (right_line, right_line)
            } else {
                (right_line + 1, right_line + additions.len())
            };
            let right_format = if right_interval.0 == right_interval.1 {
                format!("{}", right_interval.0)
            } else {
                format!("{},{}", right_interval.0, right_interval.1)
            };

            println!("{}{}{}", left_format, change_type, right_format);
            removals.iter().for_each(|s: &&String| {
                println!("< {}", config.color_config.removed.paint(s.as_str()));
            });
            if change_type == 'c' {
                println!("---");
            }
            additions.iter().for_each(|s: &&String| {
                println!("> {}", config.color_config.added.paint(s.as_str()));
            });

            left_line += removals.len();
            right_line += additions.len();
        }
    }
}

pub fn print_unified_diffs(config: &Config, changes: &[Change<&String>]) {
    if changes.is_empty() {
        return;
    }

    println!("--- {}", config.left_file);
    println!("+++ {}", config.right_file);

    let hunks = get_hunks(&changes);
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
}

/// A hunk is a contiguous subsequence of tokens from two sequences.
/// It is either a sequence of identical tokens from both sequences
/// or a combination of removals from the left sequence and additions
/// to the right sequence.
#[derive(Debug, PartialEq)]
struct Hunk<'a, T> {
    left_line: usize,
    right_line: usize,
    additions: Vec<&'a T>,
    removals: Vec<&'a T>,
    unchanged: Vec<&'a T>,
}

fn get_hunks<'a, T>(changes: &[Change<&'a T>]) -> Vec<Hunk<'a, T>> {
    let groups = changes.iter().group_by(|change| match change {
        Change::Same(_) => true,
        _ => false,
    });
    let (_, _, hunks) = groups.into_iter().fold(
        (0, 0, Vec::new()),
        |(mut left_line, mut right_line, mut hunks), (is_same, group)| {
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
                let (removals, additions): (Vec<&T>, Vec<&T>) =
                    group.partition_map(|change| match change {
                        Change::Removed(s) => Either::Left(s),
                        Change::Added(s) => Either::Right(s),
                        Change::Same(_) => panic!("Change:Same should be filtered out."),
                    });
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
            (left_line, right_line, hunks)
        },
    );
    hunks
}

#[test]
fn test_get_hunks() {
    let changes = vec![
        Change::Added(&1),
        Change::Same(&2),
        Change::Same(&3),
        Change::Removed(&4),
    ];

    let hunks: Vec<Hunk<i32>> = get_hunks(&changes);
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
}
