use crate::color::ColorConfig;
use crate::Change;

pub fn print_normal_hunks(color_config: &ColorConfig, changes: &[Change<&String>]) {
    use itertools::{Either, Itertools};

    let groups = changes.iter().group_by(|change| match change {
        Change::Same(_) => true,
        _ => false,
    });

    let mut left_line = 0;
    let mut right_line = 0;

    for (same, group) in &groups {
        if same {
            let unchanged: Vec<_> = group.map(|change| change.get()).collect();
            left_line += unchanged.len();
            right_line += unchanged.len();
        } else {
            let (removals, additions): (Vec<_>, Vec<_>) =
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
                println!("< {}", color_config.removed.paint(s.as_str()));
            });
            if change_type == 'c' {
                println!("---");
            }
            additions.iter().for_each(|s: &&String| {
                println!("> {}", color_config.added.paint(s.as_str()));
            });

            left_line += removals.len();
            right_line += additions.len();
        }
    }
}
