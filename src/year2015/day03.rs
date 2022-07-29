use rustc_hash::{FxHashSet};

common::day!(parse, part1, part2, 1000, 1000, 1000);

fn parse(data: &[u8]) -> Vec<(i32, i32)> {
    data.iter()
        .take_while(|v| **v != b'\n')
        .map(|v| match *v {
            b'<' => (-1, 0),
            b'>' => (1, 0),
            b'v' => (0, 1),
            b'^' => (0, -1),
            _ => unreachable!(),
        })
        .collect()
}

fn part1(input: &[(i32, i32)]) -> usize {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut cx = 0;
    let mut cy = 0;

    visited.insert((0, 0));
    for (xo, yo) in input.iter() {
        cx += xo;
        cy += yo;

        visited.insert((cx, cy));
    }

    visited.len()
}

fn part2(input: &[(i32, i32)]) -> usize {
    let mut visited: FxHashSet<(i32, i32)> = FxHashSet::default();
    let mut santa_x = 0;
    let mut santa_y = 0;
    let mut robot_x = 0;
    let mut robot_y = 0;
    let mut santas_turn = false;

    visited.insert((0, 0));
    for (xo, yo) in input.iter() {
        if santas_turn {
            santa_x += xo;
            santa_y += yo;

            visited.insert((santa_x, santa_y));
        } else {
            robot_x += xo;
            robot_y += yo;

            visited.insert((robot_x, robot_y));
        }

        santas_turn = !santas_turn;
    }

    visited.len()
}
