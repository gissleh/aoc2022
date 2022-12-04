use common::aoc::Day;
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
    const U128_BITS: [u128; 128] = [ // If it looks stupid, but saves µs, is it really stupid?
        1 << 0, 1 << 1, 1 << 2, 1 << 3, 1 << 4, 1 << 5, 1 << 6, 1 << 7, 1 << 8, 1 << 9, 1 << 10,
        1 << 11, 1 << 12, 1 << 13, 1 << 14, 1 << 15, 1 << 16, 1 << 17, 1 << 18, 1 << 19, 1 << 20,
        1 << 21, 1 << 22, 1 << 23, 1 << 24, 1 << 25, 1 << 26, 1 << 27, 1 << 28, 1 << 29, 1 << 30,
        1 << 31, 1 << 32, 1 << 33, 1 << 34, 1 << 35, 1 << 36, 1 << 37, 1 << 38, 1 << 39, 1 << 40,
        1 << 41, 1 << 42, 1 << 43, 1 << 44, 1 << 45, 1 << 46, 1 << 47, 1 << 48, 1 << 49, 1 << 50,
        1 << 51, 1 << 52, 1 << 53, 1 << 54, 1 << 55, 1 << 56, 1 << 57, 1 << 58, 1 << 59, 1 << 60,
        1 << 61, 1 << 62, 1 << 63, 1 << 64, 1 << 65, 1 << 66, 1 << 67, 1 << 68, 1 << 69, 1 << 70,
        1 << 71, 1 << 72, 1 << 73, 1 << 74, 1 << 75, 1 << 76, 1 << 77, 1 << 78, 1 << 79, 1 << 80,
        1 << 81, 1 << 82, 1 << 83, 1 << 84, 1 << 85, 1 << 86, 1 << 87, 1 << 88, 1 << 89, 1 << 90,
        1 << 91, 1 << 92, 1 << 93, 1 << 94, 1 << 95, 1 << 96, 1 << 97, 1 << 98, 1 << 99, 1 << 100,
        1 << 101, 1 << 102, 1 << 103, 1 << 104, 1 << 105, 1 << 106, 1 << 107, 1 << 108, 1 << 109,
        1 << 110, 1 << 111, 1 << 112, 1 << 113, 1 << 114, 1 << 115, 1 << 116, 1 << 117, 1 << 118,
        1 << 119, 1 << 120, 1 << 121, 1 << 122, 1 << 123, 1 << 124, 1 << 125, 1 << 126, 1 << 127
    ];

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

#[cfg(test)]
mod tests {
    use super::*;
}