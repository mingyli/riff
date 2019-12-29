use std::io;

use itertools::{Either, Itertools};

use crate::diff::{Change, ChangeType};

/// A hunk is a contiguous subsequence of tokens from two sequences.
/// It is either a sequence of identical tokens from both sequences
/// or a combination of removals from the left sequence and additions
/// to the right sequence. An invariant is that either `unchanged` is empty
/// or `additions` and `removals` are empty.
#[derive(Debug, PartialEq)]
pub struct Hunk<'a, T> {
    pub left_line: usize,
    pub right_line: usize,
    pub additions: Vec<&'a T>,
    pub removals: Vec<&'a T>,
    pub unchanged: Vec<&'a T>,
}

impl<'a, T> Hunk<'a, T> {
    pub fn is_same(&self) -> bool {
        !self.unchanged.is_empty()
    }

    pub fn is_addition(&self) -> bool {
        !self.additions.is_empty()
    }

    pub fn is_removal(&self) -> bool {
        !self.removals.is_empty()
    }

    pub fn is_change(&self) -> bool {
        self.is_addition() && self.is_removal()
    }

    pub fn change_type(&self) -> ChangeType {
        if self.is_change() {
            ChangeType::Changed
        } else if self.is_addition() {
            ChangeType::Added
        } else {
            ChangeType::Deleted
        }
    }

    pub fn left_interval(&self) -> (usize, usize) {
        if self.removals.is_empty() {
            (self.left_line, self.left_line)
        } else {
            (self.left_line, self.left_line + self.removals.len() - 1)
        }
    }

    pub fn right_interval(&self) -> (usize, usize) {
        if self.additions.is_empty() {
            (self.right_line, self.right_line)
        } else {
            (self.right_line, self.right_line + self.additions.len() - 1)
        }
    }
}

pub fn get_hunks<'a, T>(changes: &[Change<&'a T>]) -> io::Result<Vec<Hunk<'a, T>>> {
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
