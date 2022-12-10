use common::aoc::Day;
use common::parse3::{expect_byte, expect_bytes, Parser, signed_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Instructions", input.len());

    day.run(1, "", 10000, || part1(&input));
    day.run(2, "", 10000, || part2(&input));
}

enum Instruction {
    Noop,
    Addx(i32),
}

fn parse(data: &[u8]) -> Vec<Instruction> {
    expect_bytes(b"noop\n").map(|_| Instruction::Noop)
        .or(
            expect_bytes(b"addx ")
                .and_instead(signed_int())
                .and_discard(expect_byte(b'\n'))
                .map(Instruction::Addx)
        )
        .repeat()
        .parse(data).unwrap()
}

fn part1(input: &[Instruction]) -> i32 {
    let mut cycle = -20;
    let mut strength = 1;
    let mut sum = 0;

    for inst in input.iter().cycle() {
        if (cycle + 1) % 40 == 0 {
            #[cfg(test)] println!("{} * {} = {}", cycle + 21, strength, (cycle + 21) * strength);
            sum += (cycle + 21) * strength;
        }

        match inst {
            Instruction::Addx(x) => {
                if (cycle + 2) % 40 == 0 {
                    #[cfg(test)] println!("{} * {} = {}", cycle + 22, strength, (cycle + 22) * strength);
                    sum += (cycle + 22) * strength;
                }

                strength += *x;
                cycle += 2;
                #[cfg(test)] println!("{}: add {}", cycle, *x);
            }
            Instruction::Noop => {
                cycle += 1;
                #[cfg(test)] println!("{}: noop", cycle);
            }
        }

        if cycle > 202 {
            break;
        }
    }

    sum
}

fn part2(input: &[Instruction]) -> String {
    let mut cycle = 0usize;
    let mut sprite_x = 1;
    let mut res = String::with_capacity(41 * 6);

    for inst in input.iter().cycle() {
        if cycle == (40 * 6) {
            break;
        }
        p2_draw_pixel(&mut res, cycle, sprite_x);

        match inst {
            Instruction::Addx(x) => {
                cycle += 1;
                if cycle == (40 * 6) {
                    break;
                }
                p2_draw_pixel(&mut res, cycle, sprite_x);

                sprite_x += *x;
                cycle += 1;
            }
            Instruction::Noop => {
                cycle += 1;
            }
        }
    }

    res
}

fn p2_draw_pixel(res: &mut String, cycle: usize, sprite_x: i32) {
    let line_x = (cycle % 40) as i32;

    if cycle % 40 == 0 && res.len() > 0 {
        res.push('\n');
    }
    if line_x >= sprite_x - 1 && line_x <= sprite_x + 1 {
        res.push('#');
    } else {
        res.push('.');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = b"addx 15\naddx -11\naddx 6\naddx -3\naddx 5\naddx -1\naddx -8\naddx 13
addx 4\nnoop\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx 5\naddx -1\naddx -35
addx 1\naddx 24\naddx -19\naddx 1\naddx 16\naddx -11\nnoop\nnoop\naddx 21\naddx -15\nnoop\nnoop
addx -3\naddx 9\naddx 1\naddx -3\naddx 8\naddx 1\naddx 5\nnoop\nnoop\nnoop\nnoop\nnoop\naddx -36
noop\naddx 1\naddx 7\nnoop\nnoop\nnoop\naddx 2\naddx 6\nnoop\nnoop\nnoop\nnoop\nnoop\naddx 1\nnoop
noop\naddx 7\naddx 1\nnoop\naddx -13\naddx 13\naddx 7\nnoop\naddx 1\naddx -33\nnoop\nnoop\nnoop\naddx 2
noop\nnoop\nnoop\naddx 8\nnoop\naddx -1\naddx 2\naddx 1\nnoop\naddx 17\naddx -9\naddx 1\naddx 1
addx -3\naddx 11\nnoop\nnoop\naddx 1\nnoop\naddx 1\nnoop\nnoop\naddx -13\naddx -19\naddx 1\naddx 3
addx 26\naddx -30\naddx 12\naddx -1\naddx 3\naddx 1\nnoop\nnoop\nnoop\naddx -9\naddx 18\naddx 1\naddx 2
noop\nnoop\naddx 9\nnoop\nnoop\nnoop\naddx -1\naddx 2\naddx -37\naddx 1\naddx 3\nnoop\naddx 15
addx -21\naddx 22\naddx -6\naddx 1\nnoop\naddx 2\naddx 1\nnoop\naddx -10\nnoop\nnoop\naddx 20\naddx 1
addx 2\naddx 2\naddx -6\naddx -11\nnoop\nnoop\nnoop\n";

    const P2_EXAMPLE_RESULT: &str = "##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....";

    #[test]
    fn p1_works_on_example() {
        let input = parse(P1_EXAMPLE);

        assert_eq!(part1(&input), 13140);
    }

    #[test]
    fn p2_works_on_example() {
        let input = parse(P1_EXAMPLE);

        assert_eq!(part2(&input).as_str(), P2_EXAMPLE_RESULT);
    }
}