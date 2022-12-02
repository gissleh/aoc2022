use common::aoc::Day;
use common::parse3;
use common::parse3::Parser;

#[inline]
fn get_strategy(opp: u8, outcome: u8) -> u8 {
    (opp + 2 + outcome) % 3
}

fn get_score(opp: u8, you: u8) -> u32 {
    const SCORES: &'static [u32] = &[
        4, 8, 3, // A -> X Y Z
        1, 5, 9, // B -> X Y Z
        7, 2, 6, // C -> X Y Z
    ];

    SCORES[((opp * 3) + you) as usize]
}

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Matches", input.len());

    day.run(1, "", 10000, || part1(&input));
    day.run(2, "", 10000, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<(u8, u8)> {
    parse3::any_byte()
        .and_skip(parse3::expect_byte(b' '))
        .and(parse3::any_byte())
        .map(|(a,b)| (a - b'A', b - b'X') )
        .and_skip(parse3::expect_byte(b'\n'))
        .repeat()
        .parse(data).unwrap()
}

fn part1(input: &[(u8, u8)]) -> u32 {
    input.iter().cloned()
        .map(|(opp, you)| get_score(opp, you))
        .sum()
}

fn part2(input: &[(u8, u8)]) -> u32 {
    input.iter().cloned()
        .map(|(opp, outcome)| get_score(opp, get_strategy(opp, outcome)))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_works_on_example() {
        let input = parse(b"A Y\nB X\nC Z\n");
        assert_eq!(part1(&input), 15);
    }

    #[test]
    fn part2_works_on_example() {
        let input = parse(b"A Y\nB X\nC Z\n");
        assert_eq!(part2(&input), 12    );
    }
}