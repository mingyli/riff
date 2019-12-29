use std::collections::VecDeque;
use std::ops::{Index, IndexMut};

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

#[derive(Debug, PartialEq)]
pub enum ChangeType {
    Deleted,
    Added,
    Changed,
}

use std::fmt;
impl fmt::Display for ChangeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChangeType::Deleted => write!(f, "d"),
            ChangeType::Added => write!(f, "a"),
            ChangeType::Changed => write!(f, "c"),
        }
    }
}

// TODO: Write this function using Rust idioms instead of relying on indexing operations.
fn longest_common_subsequence<'a, T>(left: &'a [T], right: &'a [T]) -> Matrix<u32>
where
    T: PartialEq,
{
    use std::cmp;

    let mut lengths: Matrix<u32> = Matrix::new(left.len() + 1, right.len() + 1);
    for (l, r) in (0..=left.len()).cartesian_product(0..=right.len()) {
        lengths[(l, r)] = if l == 0 || r == 0 {
            0
        } else if left[l - 1] == right[r - 1] {
            lengths[(l - 1, r - 1)] + 1
        } else {
            cmp::max(lengths[(l, r - 1)], lengths[(l - 1, r)])
        }
    }
    lengths
}

// TODO: Write this function using Rust idioms instead of relying on indexing operations.
pub fn diff<'a, T>(left: &'a [T], right: &'a [T]) -> Vec<Change<&'a T>>
where
    T: PartialEq,
{
    if left.is_empty() || right.is_empty() {
        return left
            .iter()
            .map(Change::Removed)
            .chain(right.iter().map(Change::Added))
            .collect();
    }

    let lengths = longest_common_subsequence(left, right);

    let mut result = VecDeque::new();
    let mut pos = (left.len(), right.len());
    while lengths[pos] > 0 {
        let (l, r) = pos;

        if lengths[(l - 1, r)] == lengths[pos] - 1
            && lengths[(l, r - 1)] == lengths[pos] - 1
            && lengths[(l - 1, r - 1)] == lengths[pos] - 1
        {
            result.push_front(Change::Same(&left[l - 1]));
            pos = (l - 1, r - 1);
        } else if lengths[(l, r - 1)] < lengths[(l - 1, r)] {
            result.push_front(Change::Removed(&left[l - 1]));
            pos = (l - 1, r);
        } else {
            result.push_front(Change::Added(&right[r - 1]));
            pos = (l, r - 1);
        }
    }
    while pos != (0, 0) {
        let (l, r) = pos;
        if l > 0 {
            result.push_front(Change::Removed(&left[l - 1]));
            pos = (l - 1, r);
        } else {
            result.push_front(Change::Added(&right[r - 1]));
            pos = (l, r - 1);
        }
    }

    Vec::from(result)
}

#[test]
fn test_diff_empty() {
    let left = [0u32; 0];
    let right = [0u32; 0];

    let changes = diff(&left, &right);
    assert_eq!(changes, vec![]);
}

#[test]
fn test_diff_same() {
    let left = [0, 1, 2];
    let right = [0, 1, 2];

    let changes = diff(&left, &right);
    assert_eq!(
        changes,
        vec![Change::Same(&0), Change::Same(&1), Change::Same(&2)]
    );
}

#[test]
fn test_diff_added() {
    let left = [];
    let right = [0, 1, 2];

    let changes = diff(&left, &right);
    assert_eq!(
        changes,
        vec![Change::Added(&0), Change::Added(&1), Change::Added(&2),]
    );
}

#[test]
fn test_diff_removed() {
    let left = [0, 1, 2];
    let right = [];

    let changes = diff(&left, &right);
    assert_eq!(
        changes,
        vec![
            Change::Removed(&0),
            Change::Removed(&1),
            Change::Removed(&2),
        ]
    );
}

#[test]
fn test_different() {
    let left = [1, 2, 3];
    let right = [2, 3, 4];

    let changes = diff(&left, &right);
    assert_eq!(
        changes,
        vec![
            Change::Removed(&1),
            Change::Same(&2),
            Change::Same(&3),
            Change::Added(&4),
        ]
    );
}

#[test]
fn test_different_reversed() {
    let left = [2, 3, 4];
    let right = [1, 2, 3];

    let changes = diff(&left, &right);
    assert_eq!(
        changes,
        vec![
            Change::Added(&1),
            Change::Same(&2),
            Change::Same(&3),
            Change::Removed(&4),
        ]
    );
}

#[test]
fn test_diff_modified() {
    let left = ["hey", "my", "name", "is"];
    let right = ["hey", "mister", "nae", "say"];

    let changes = diff(&left, &right);
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
}

#[test]
fn test_diff_mixed() {
    let left = ["sphinx", "of", "black", "quartz", "judge", "my", "vow"];
    let right = ["sphinx", "offer", "black", "quartz", "my", "jughead", "vow"];

    let changes = diff(&left, &right);
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
}
