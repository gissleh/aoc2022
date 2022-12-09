use common::aoc::Day;
use common::constants::U128_BITS;
use common::parse3;
use common::parse3::Parser;

pub fn main(day: &mut Day, input_data: &[u8]) {
    let input = day.run_parse_labeled("Mask", 1000, || parse(input_data));
    let input_ranges = day.run_parse_labeled("Ranges", 1000, || parse_ranges(input_data));

    day.note("Pairs", input.len());

    day.run(1, "Mask", 10000, || part1(&input));
    day.run(1, "Ranges", 10000, || part1_ranges(&input_ranges));
    day.run(2, "Mask", 10000, || part2(&input));
    day.run(2, "Ranges", 10000, || part2_ranges(&input_ranges));

    day.select_label("Mask");
}

fn parse(data: &[u8]) -> Vec<(u128, u128)> {
    fn get_bitmask(low: u32, high: u32) -> u128 {
        // Credit: https://www.reddit.com/r/adventofcode/comments/zc0zta/comment/iyvrqwm/?utm_source=share&utm_medium=web2x&context=3
        assert!(low < 128 && high < 128);
        let high = U128_BITS[high as usize] - 1;
        let low = U128_BITS[(low - 1) as usize] - 1;
        high - low
    }

    parse3::unsigned_int()
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b','))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b'\n'))
        .map(|(((a1, a2), b1), b2)| (
            get_bitmask(a1, a2),
            get_bitmask(b1, b2),
        ))
        .repeat()
        .parse(data).unwrap()
}

struct Work(u8, u8);

impl Work {
    fn contains(&self, other: &Self) -> bool {
        self.0 >= other.0 && self.1 <= other.1
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.0 <= other.1 && self.1 >= other.0
    }
}

fn parse_ranges(data: &[u8]) -> Vec<(Work, Work)> {
    parse3::unsigned_int()
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b','))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int())
        .and_discard(parse3::expect_byte(b'\n'))
        .map(|(((a1, a2), b1), b2)| (Work(a1, a2), Work(b1, b2)))
        .repeat()
        .parse(data).unwrap()
}

fn part1(input: &[(u128, u128)]) -> usize {
    input.iter()
        .filter(|(elf1, elf2)| {
            let overlap = *elf1 & *elf2;
            *elf1 == overlap || *elf2 == overlap
        })
        .count()
}

fn part1_ranges(input: &[(Work, Work)]) -> usize {
    input.iter()
        .filter(|(a, b)| a.contains(b) || b.contains(a))
        .count()
}

fn part2(input: &[(u128, u128)]) -> usize {
    input.iter()
        .filter(|(elf1, elf2)| *elf1 & *elf2 > 0)
        .count()
}

fn part2_ranges(input: &[(Work, Work)]) -> usize {
    input.iter()
        .filter(|(elf1, elf2)| elf1.overlaps(elf2))
        .count()
}
