use std::collections::VecDeque;
use std::io;

use itertools::Itertools;

type Position = (usize, usize);

#[derive(Debug)]
struct Matrix<T> {
    data: Vec<T>,
    height: usize,
    width: usize,
}

impl<T> Matrix<T>
where
    T: Clone + Default,
{
    fn new(height: usize, width: usize) -> Matrix<T> {
        Matrix {
            data: vec![Default::default(); height * width],
            height,
            width,
        }
    }
}

use std::ops::{Index, IndexMut};
impl<T> Index<Position> for Matrix<T>
where
    T: Clone,
{
    type Output = T;

    fn index(&self, (i, j): Position) -> &Self::Output {
        &self.data[i * self.width + j]
    }
}
impl<T> IndexMut<Position> for Matrix<T>
where
    T: Clone,
{
    fn index_mut(&mut self, (i, j): Position) -> &mut Self::Output {
        &mut self.data[i * self.width + j]
    }
}

#[derive(Debug, PartialEq)]
pub enum Change<T> {
    Removed(T),
    Added(T),
    Same(T),
}

impl<T> Change<T> {
    pub fn get(&self) -> &T {
        match self {
            Change::Removed(t) => t,
            Change::Added(t) => t,
            Change::Same(t) => t,
        }
    }
}

fn longest_common_subsequence<'a, T>(
    left: &'a [T],
    right: &'a [T],
) -> (Matrix<u32>, Matrix<Position>)
where
    T: PartialEq,
{
    use std::cmp;

    let mut lengths: Matrix<u32> = Matrix::new(left.len(), right.len());
    let mut backtracks: Matrix<(usize, usize)> = Matrix::new(left.len(), right.len());
    for (l, r) in (0..left.len()).cartesian_product(0..right.len()) {
        let same = left[l] == right[r];

        lengths[(l, r)] = if (l, r) == (0, 0) {
            same as u32
        } else if l == 0 {
            cmp::max(same as u32, lengths[(l, r - 1)])
        } else if r == 0 {
            cmp::max(same as u32, lengths[(l - 1, r)])
        } else if left[l] == right[r] {
            lengths[(l - 1, r - 1)] + 1
        } else {
            cmp::max(lengths[(l, r - 1)], lengths[(l - 1, r)])
        };

        backtracks[(l, r)] = if (l, r) == (0, 0) {
            (l, r)
        } else if l == 0 {
            (l, r - 1)
        } else if r == 0 {
            (l - 1, r)
        } else if left[l] == right[r] {
            (l - 1, r - 1)
        } else {
            *[(l - 1, r), (l, r - 1)]
                .iter()
                .max_by_key(|&&pos| lengths[pos])
                .expect("There are two positions.")
        }
    }
    (lengths, backtracks)
}

pub fn diff<'a, T>(left: &'a [T], right: &'a [T]) -> io::Result<Vec<Change<&'a T>>>
where
    T: PartialEq,
{
    if left.is_empty() || right.is_empty() {
        return Ok(left
            .iter()
            .map(Change::Removed)
            .chain(right.iter().map(Change::Added))
            .collect());
    }

    let (_lengths, backtracks) = longest_common_subsequence(left, right);

    let mut result = VecDeque::new();
    let mut pos: Position = (left.len() - 1, right.len() - 1);
    loop {
        let (i, j) = pos;
        let (bi, bj) = backtracks[pos];

        if pos == (0, 0) {
            if left[0] == right[0] {
                result.push_front(Change::Same(&left[0]));
            } else {
                result.push_front(Change::Removed(&left[0]));
                result.push_front(Change::Added(&right[0]));
            }
            break;
        } else if bi + 1 == i && bj + 1 == j {
            result.push_front(Change::Same(&left[i]));
        } else if bi + 1 == i {
            result.push_front(Change::Removed(&left[i]));
        } else if bj + 1 == j {
            result.push_front(Change::Added(&right[j]));
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Backtracking failed.",
            ));
        }

        pos = backtracks[pos];
    }

    Ok(Vec::from(result))
}

#[test]
fn test_diff_empty() -> io::Result<()> {
    let left = [0u32; 0];
    let right = [0u32; 0];

    let changes = diff(&left, &right)?;
    assert_eq!(changes, vec![]);
    Ok(())
}

#[test]
fn test_diff_same() -> io::Result<()> {
    let left = [0, 1, 2];
    let right = [0, 1, 2];

    let changes = diff(&left, &right)?;
    assert_eq!(
        changes,
        vec![Change::Same(&0), Change::Same(&1), Change::Same(&2)]
    );
    Ok(())
}

#[test]
fn test_diff_added() -> io::Result<()> {
    let left = [];
    let right = [0, 1, 2];

    let changes = diff(&left, &right)?;
    assert_eq!(
        changes,
        vec![Change::Added(&0), Change::Added(&1), Change::Added(&2),]
    );
    Ok(())
}

#[test]
fn test_diff_removed() -> io::Result<()> {
    let left = [0, 1, 2];
    let right = [];

    let changes = diff(&left, &right)?;
    assert_eq!(
        changes,
        vec![
            Change::Removed(&0),
            Change::Removed(&1),
            Change::Removed(&2),
        ]
    );
    Ok(())
}

// TODO: Fix this test.
// #[test]
// fn test_different() -> io::Result<()> {
//     let left = [1, 2, 5, 6];
//     let right = [3, 4, 7, 8];
//
//     let changes = diff(&left, &right)?;
//     assert_eq!(changes, vec![]);
//     Ok(())
// }

#[test]
fn test_diff_modified() -> io::Result<()> {
    let left = ["hey", "my", "name", "is"];
    let right = ["hey", "mister", "nae", "say"];

    let changes = diff(&left, &right)?;
    assert_eq!(
        changes,
        vec![
            Change::Same(&"hey"),
            Change::Removed(&"my"),
            Change::Removed(&"name"),
            Change::Removed(&"is"),
            Change::Added(&"mister"),
            Change::Added(&"nae"),
            Change::Added(&"say"),
        ]
    );
    Ok(())
}

#[test]
fn test_diff_mixed() -> io::Result<()> {
    let left = ["sphinx", "of", "black", "quartz", "judge", "my", "vow"];
    let right = ["sphinx", "offer", "black", "quartz", "my", "jughead", "vow"];

    let changes = diff(&left, &right)?;
    assert_eq!(
        changes,
        vec![
            Change::Same(&"sphinx"),
            Change::Removed(&"of"),
            Change::Added(&"offer"),
            Change::Same(&"black"),
            Change::Same(&"quartz"),
            Change::Removed(&"judge"),
            Change::Same(&"my"),
            Change::Added(&"jughead"),
            Change::Same(&"vow"),
        ]
    );
    Ok(())
}
