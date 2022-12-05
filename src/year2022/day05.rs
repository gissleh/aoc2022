use common::aoc::Day;
use common::parse3;
use common::parse3::Parser;

pub fn main(day: &mut Day, input: &[u8]) {
    let (stacks, moves) = day.run_parse(1000, || parse(input));

    day.note("stacks", stacks.len());
    day.note("Boxes", stacks.iter().map(|v| v.len()).sum::<usize>());
    day.note("Moves", moves.len());

    day.run(1, "", 10000, || part1(&stacks, &moves));
    day.run(2, "", 10000, || part2(&stacks, &moves));
}

fn parse(data: &[u8]) -> (Vec<Vec<u8>>, Vec<(u8, u8, u8)>) {
    let line_len = (parse3::line().parse(data).unwrap().len() + 1) / 4;
    let (crane_states, data) = parse3::any_byte()
        .quoted_by(b'[', b']')
        .map(|c| Some(c))
        .or(parse3::expect_bytes(b"   ").map(|_| None))
        .and_discard(
            parse3::expect_byte(b'\n')
                .or(parse3::expect_byte(b' '))
        )
        .repeat()
        .parse(data).unwrap_and_input();

    let mut stacks = vec![Vec::with_capacity(26); line_len];
    for (i, crane_state) in crane_states.iter().enumerate() {
        if let Some(state) = crane_state {
            stacks[i % line_len].push(*state);
        }
    }
    for crane in stacks.iter_mut() {
        crane.reverse();
    }

    let paragraph_pos = data.array_windows::<2>().take_while(|v| **v != [b'\n', b'\n']).count();

    assert!(paragraph_pos + 2 < data.len());

    let moves = parse3::expect_bytes(b"move ")
        .and_instead(parse3::unsigned_int::<u8>())
        .and_discard(parse3::expect_bytes(b" from "))
        .and(parse3::unsigned_int::<u8>())
        .and_discard(parse3::expect_bytes(b" to "))
        .and(parse3::unsigned_int::<u8>())
        .skip(parse3::expect_byte(b'\n'))
        .map(|((a, b), c)| (a, b - 1, c - 1))
        .repeat()
        .parse(&data[paragraph_pos + 2..]).unwrap();

    (stacks, moves)
}

fn part1(stacks: &[Vec<u8>], moves: &[(u8, u8, u8)]) -> String {
    let mut stacks = stacks.to_vec();

    for (c, from, to) in moves.iter().cloned() {
        for _ in 0..c {
            let top_crate = stacks[from as usize].pop().unwrap();
            stacks[to as usize].push(top_crate);
        }
    }

    stack_top_string(&stacks)
}

fn part2(stacks: &[Vec<u8>], moves: &[(u8, u8, u8)]) -> String {
    let mut stacks = stacks.to_vec();

    for (c, from, to) in moves.iter().cloned() {
        let si = stacks[from as usize].len() - c as usize;

        for i in 0..c as usize {
            let top_crate = stacks[from as usize][si + i];
            stacks[to as usize].push(top_crate);
        }

        stacks[from as usize].truncate(si);
    }

    stack_top_string(&stacks)
}

fn stack_top_string(stacks: &Vec<Vec<u8>>) -> String {
    let v: Vec<u8> = stacks.iter()
        .map(|v| v.last().cloned())
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .collect();

    String::from_utf8_lossy(&v).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &'static [u8] = b"    [D]    \n[N] [C]    \n[Z] [M] [P]\n 1   2   3 \n
move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

    #[test]
    fn parse_works_on_example() {
        let (stacks, moves) = parse(EXAMPLE);

        assert_eq!(&stacks, &[b"ZN".to_vec(), b"MCD".to_vec(), b"P".to_vec()]);
        assert_eq!(&moves, &[(1, 1, 0), (3, 0, 2), (2, 1, 0), (1, 0, 1)])
    }

    #[test]
    fn part1_works_on_example() {
        let (stacks, moves) = parse(EXAMPLE);
        assert_eq!(part1(&stacks, &moves), String::from("CMZ"));
    }

    #[test]
    fn part2_works_on_example() {
        let (stacks, moves) = parse(EXAMPLE);
        assert_eq!(part2(&stacks, &moves), String::from("MCD"));
    }
}