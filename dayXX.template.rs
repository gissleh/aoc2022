common::day!(parse, part1, part2, 10000, 10, 10);

fn parse(data: &[u8]) -> Vec<u8> {
    data.iter().take_while(|b| **b != b'\n').cloned().collect()
}

fn part1(input: &[u8]) -> usize {
    0
}

fn part2(input: &[u8]) -> usize {
    0
}

#[cfg(test)]
mod tests {
    use super::*;
}