use common::aoc::Day;
use common::parse3;
use common::parse3::Parser;

pub fn main(day: &mut Day, input_data: &[u8]) {
    let input = day.run_parse(1000, || parse(input_data));
    let input_mask = day.run_parse_labeled("Mask", 1000, || parse_mask(input_data));

    day.note("Backpacks", input.len());

    day.run(1, "", 10000, || part1(&input));
    day.run(1, "Mask", 10000, || part1_mask(&input_mask));
    day.run(1, "Mask [does not work if two items overlap]", 10000, || part1_mask_naughty(&input_mask));
    day.run(2, "", 10000, || part2(&input));
    day.run(2, "Mask", 10000, || part2_mask(&input_mask));
}

fn index(ch: &u8) -> usize {
    if b'a' <= *ch && *ch <= b'z' {
        (ch - b'a') as usize
    } else {
        (ch - b'A') as usize + 26
    }
}

fn parse(data: &[u8]) -> Vec<([u16; 52], [u16; 52])> {
    parse3::line()
        .map(|l| (
            l[..l.len() / 2].iter().fold([0u16; 52], |mut a, c| {
                a[index(c)] += 1;
                a
            }),
            l[l.len() / 2..].iter().fold([0u16; 52], |mut a, c| {
                a[index(c)] += 1;
                a
            }),
        ))
        .repeat()
        .parse(data).unwrap()
}

fn parse_mask(data: &[u8]) -> Vec<(u64, u64)> {
    parse3::line()
        .map(|l| (
            l[..l.len() / 2].iter().fold(0u64, |a, c| a | (1 << index(c) as u64)),
            l[l.len() / 2..].iter().fold(0u64, |a, c| a | (1 << index(c) as u64)),
        ))
        .repeat()
        .parse(data).unwrap()
}

fn part1(input: &[([u16; 52], [u16; 52])]) -> u32 {
    let mut acc = [0u32; 52];
    for (a, b) in input {
        for (i, (a, b)) in a.iter().zip(b.iter()).enumerate() {
            if *a > 0 && *b > 0 {
                acc[i] += (i as u32) + 1;
            }
        }
    }

    acc.iter().sum()
}

fn part1_mask(input: &[(u64, u64)]) -> u32 {
    input.iter()
        .map(|(a, b)| *a & *b)
        .filter(|v| *v != 0)
        .fold(0, |mut acc, mut v| {
            let mut n = v.trailing_zeros();
            while n < 64 {
                acc += n + 1;
                v -= 1 << n;

                n = v.trailing_zeros();
            }

            acc
        })
}

fn part1_mask_naughty(input: &[(u64, u64)]) -> u32 {
    input.iter()
        .map(|(a, b)| *a & *b)
        .filter(|v| *v != 0)
        .fold(0, |acc, v| acc + v.trailing_zeros() + 1)
}

fn part2(input: &[([u16; 52], [u16; 52])]) -> u32 {
    let mut sum = 0;
    for group_sacks in input.array_chunks::<3>() {
        let mut found_list = [[false; 52]; 3];

        for (i, (a, b)) in group_sacks.iter().enumerate() {
            for (j, (a, b)) in a.iter().zip(b.iter()).enumerate() {
                if *a > 0 || *b > 0 {
                    found_list[i][j] = true;
                }
            }
        }

        sum += found_list[0].iter()
            .zip(found_list[1].iter())
            .zip(found_list[2].iter())
            .take_while(|((a, b), c)| !**a || !**b || !**c)
            .count() as u32 + 1;
    }

    sum
}

fn part2_mask(input: &[(u64, u64)]) -> u32 {
    input.array_chunks::<3>()
        .map(|[(a1,a2), (b1, b2), (c1, c2)]| {
            ((*a1 | *a2) & (*b1 | *b2) & (*c1 | *c2)).trailing_zeros() + 1
        })
        .sum()
}
