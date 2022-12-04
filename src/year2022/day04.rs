use common::aoc::Day;
use common::parse3;
use common::parse3::Parser;

pub fn main(day: &mut Day, input_data: &[u8]) {
    let input = day.run_parse_labeled("Mask", 1000, || parse(input_data));
    day.run_parse_labeled("Mask [w/o parser3]", 1000, || parse_raw(input_data));
    let input_ranges = day.run_parse_labeled("Ranges", 1000, || parse_ranges(input_data));

    day.run(1, "Mask", 10000, || part1(&input));
    day.run(1, "Ranges", 10000, || part1_ranges(&input_ranges));
    day.run(2, "Mask", 10000, || part2(&input));
    day.run(2, "Ranges", 10000, || part2_ranges(&input_ranges));

    day.select_label("Ranges");
}

fn parse(data: &[u8]) -> Vec<(u128, u128)> {
    parse3::unsigned_int::<u32>()
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int::<u32>())
        .and_discard(parse3::expect_byte(b','))
        .and(parse3::unsigned_int::<u32>())
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int::<u32>())
        .and_discard(parse3::expect_byte(b'\n'))
        .map(|(((a1, a2), b1), b2)| (
            (a1..=a2).fold(0u128, |p, c| p | 1 << c),
            (b1..=b2).fold(0u128, |p, c| p | 1 << c),
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
    parse3::unsigned_int::<u8>()
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int::<u8>())
        .and_discard(parse3::expect_byte(b','))
        .and(parse3::unsigned_int::<u8>())
        .and_discard(parse3::expect_byte(b'-'))
        .and(parse3::unsigned_int::<u8>())
        .and_discard(parse3::expect_byte(b'\n'))
        .map(|(((a1, a2), b1), b2)| (Work(a1, a2), Work(b1, b2)))
        .repeat()
        .parse(data).unwrap()
}

fn parse_raw(data: &[u8]) -> Vec<(u128, u128)> {
    let mut res = Vec::with_capacity(128);
    let mut curr = [0u32; 4];
    let mut i = 0usize;
    for c in data {
        match *c {
            b'0'..=b'9' => {
                curr[i] *= 10;
                curr[i] += (*c - b'0') as u32
            }
            b',' | b'-' => {
                i += 1;
                curr[i] = 0;
            }
            b'\n' if i == 3 => {
                res.push((
                    (curr[0]..=curr[1]).fold(0u128, |p, c| p | 1 << c),
                    (curr[2]..=curr[3]).fold(0u128, |p, c| p | 1 << c),
                ));
                i = 0;
                curr[0] = 0;
            }
            _ => {}
        }
    }

    res
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
        .filter(|(a, b)| a.overlaps(b))
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
}