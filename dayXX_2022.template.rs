use common::aoc::Day;

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.run(1, "", 10000, || part1(&input));
    day.run(2, "", 10000, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

fn part1(input: &[u8]) -> u32 {
    0
}

fn part2(input: &[u8]) -> u32 {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
}