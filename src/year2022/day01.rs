use common::aoc::Day;
use common::parse2;

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.run(1, "", 1000, || part1(&input));
    day.run(2, "", 1000, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<Vec<u32>> {
    parse2::map(data, |input| parse2::paragraph(input).map(|paragraph| {
        parse2::map(paragraph, |input| {
            parse2::uint::<u32>(input).and_discard(parse2::skip_byte::<b'\n'>)
        }).collect()
    })).collect()
}

fn part1(input: &[Vec<u32>]) -> u32 {
    input.iter()
        .map(|a| a.iter().sum::<u32>())
        .max()
        .unwrap()
}

fn part2(input: &[Vec<u32>]) -> u32 {
    let mut max = [0u32; 3];
    for calories in input.iter().map(|a| a.iter().sum::<u32>()) {
        for i in 0..max.len() {
            if calories > max[i] {
                max[i] = calories;
                max.sort();
                break;
            }
        }
    }

    max.iter().sum()
}

#[cfg(test)]
mod tests {
    use super::*;
}