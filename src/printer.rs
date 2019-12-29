use crate::Change;

pub fn print_hunks(changes: &[Change<&String>]) {
    use ansi_term::Colour;
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
            left_line += removals.len();
            right_line += additions.len();
            println!("{}{}{}", left_line, change_type, right_line);
            removals.iter().for_each(|s: &&String| {
                println!("< {}", Colour::Black.on(Colour::Red).paint(s.as_str()));
            });
            if change_type == 'c' {
                println!("---");
            }
            additions.iter().for_each(|s: &&String| {
                println!("> {}", Colour::Black.on(Colour::Green).paint(s.as_str()));
            });
        }
    }
}
