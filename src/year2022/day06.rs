use common::aoc::Day;

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Input length", input.len());

    day.run(1, "", 10000, || part1(&input));
    day.run(1, "Linear", 10000, || part1_linear(&input));
    day.run(2, "", 10000, || part2(&input));
    day.run(2, "Linear", 10000, || part2_linear(&input));
}

fn parse(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

fn part1(input: &[u8]) -> usize {
    input.array_windows::<4>()
        .cloned()
        .take_while(|[a, b, c, d]| {
            a == b || a == c || a == d
                || b == c || b == d
                || c == d
        })
        .count() + 4
}

fn part1_linear(input: &[u8]) -> usize { part_linear::<5>(input) }

fn part2_linear(input: &[u8]) -> usize { part_linear::<15>(input) }

fn part_linear<const L: usize>(input: &[u8]) -> usize {
    let target_len = L - 1;
    let mut uniques = 0;
    let mut char_count = [0u32; 26];
    for ch in input.iter().take(target_len) {
        let i = (*ch - b'a') as usize;
        if char_count[i] == 0 {
            uniques += 1;
        }
        char_count[i] += 1;
    }
    if uniques == target_len {
        return target_len;
    }

    let mut count = L;
    for a in input.array_windows::<L>() {
        let new = (a[target_len] - b'a') as usize;
        let old = (a[0] - b'a') as usize;

        char_count[new] += 1;
        if char_count[new] == 1 { uniques += 1; }

        char_count[old] -= 1;
        if char_count[old] == 0 { uniques -= 1; }

        if uniques == target_len {
            break;
        }

        count += 1;
    }

    count
}

fn part2(input: &[u8]) -> usize {
    input.array_windows::<14>()
        .cloned()
        .take_while(|a| {
            for i in 0..14 {
                for j in 0..i {
                    if a[i] == a[j] {
                        return true;
                    }
                }
            }

            false
        })
        .count() + 14
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn part1_works_on_examples() {
        assert_eq!(part1(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 5);
        assert_eq!(part1(b"nppdvjthqldpwncqszvftbrmjlhg"), 6);
        assert_eq!(part1(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 10);
        assert_eq!(part1(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 11);
    }

    #[test]
    fn part_mask_works_on_examples() {
        assert_eq!(part_linear::<5>(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 5);
        assert_eq!(part_linear::<5>(b"nppdvjthqldpwncqszvftbrmjlhg"), 6);
        assert_eq!(part_linear::<5>(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 10);
        assert_eq!(part_linear::<5>(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 11);
        assert_eq!(part_linear::<15>(b"mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 19);
        assert_eq!(part_linear::<15>(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 23);
        assert_eq!(part_linear::<15>(b"nppdvjthqldpwncqszvftbrmjlhg"), 23);
        assert_eq!(part_linear::<15>(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 29);
        assert_eq!(part_linear::<15>(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 26);
    }

    #[test]
    fn part2_works_on_examples() {
        assert_eq!(part2(b"mjqjpqmgbljsphdztnvjfqwrcgsmlb"), 19);
        assert_eq!(part2(b"bvwbjplbgvbhsrlpgdmjqwftvncz"), 23);
        assert_eq!(part2(b"nppdvjthqldpwncqszvftbrmjlhg"), 23);
        assert_eq!(part2(b"nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), 29);
        assert_eq!(part2(b"zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), 26);
    }
}