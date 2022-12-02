use common::aoc::Day;
use common::parse3;
use common::parse3::Parser;

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Elves", input.len());
    day.note("Rations", input.iter().map(|v| v.len()).sum::<usize>());
    day.note("Calories", input.iter().map(|v| v.iter().sum::<u32>()).sum::<u32>());

    day.run(1, "", 10000, || part1(&input));
    day.run(2, "", 10000, || part2(&input));
    day.run(2, "Fold", 10000, || part2_fold(&input));
}

fn parse(data: &[u8]) -> Vec<Vec<u32>> {
    parse3::unsigned_int::<u32>()
        .and_skip(parse3::expect_byte(b'\n'))
        .repeat_until(parse3::expect_byte(b'\n'))
        .repeat()
        .parse(data).unwrap()
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
                max.sort_unstable();
                break;
            }
        }
    }

    max.iter().sum()
}

fn part2_fold(input: &[Vec<u32>]) -> u32 {
    input.iter()
        .map(|a| a.iter().sum::<u32>())
        .fold([0, 0, 0], |a, current| {
            if current >= a[0] {
                [current, a[0], a[1]]
            } else if current >= a[1] {
                [a[0], current, a[1]]
            } else if current > a[2] {
                [a[0], a[1], current]
            } else {
                a
            }
        })
        .iter()
        .sum()
}
