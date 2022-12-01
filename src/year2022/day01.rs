common::day!(parse, part1, part2, 10000, 10000, 10000);

use common::parse2;

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