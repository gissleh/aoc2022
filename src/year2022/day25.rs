use common::aoc::Day;
use common::parse3::{line, Parser};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(100000, || parse(input));

    day.note("Numbers", input.len());
    day.note("Sum Base-10", input.iter().sum::<i64>());

    day.run(1, "", 100000, || part1(&input));
}

fn parse(data: &[u8]) -> Vec<i64> {
    line().map(|l| parse_snafu(l)).repeat().parse(data).unwrap()
}

fn part1(input: &[i64]) -> String {
    into_snafu(input.iter().sum())
}

fn into_snafu(v: i64) -> String {
    let mut data = [b'0'; 32];
    let mut pos = data.len();
    let mut v = v;

    while v > 0 {
        pos -= 1;

        let digit_base5 = v % 5;
        v /= 5;

        let ch = match digit_base5 {
            0..=2 => b'0' + (digit_base5 as u8),
            3 => {
                v += 1;
                b'='
            }
            4 => {
                v += 1;
                b'-'
            }
            _ => unreachable!()
        };

        data[pos] = ch;
    }

    String::from_utf8_lossy(&data[pos..]).to_string()
}

fn parse_snafu(v: &[u8]) -> i64 {
    let mut res = 0;
    for ch in v {
        let v = match *ch {
            b'-' => -1,
            b'=' => -2,
            _ => (*ch - b'0') as i64
        };

        res *= 5;
        res += v;
    }

    res
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snafu_works() {
        assert_eq!(into_snafu(1), "1");
        assert_eq!(into_snafu(2), "2");
        assert_eq!(into_snafu(3), "1=");
        assert_eq!(into_snafu(4), "1-");
        assert_eq!(into_snafu(5), "10");
        assert_eq!(into_snafu(314159265), "1121-1110-1=0");
        assert_eq!(into_snafu(4890), "2=-1=0");
    }

    #[test]
    fn parses_snafu_right() {
        assert_eq!(parse_snafu(b"1"), 1);
        assert_eq!(parse_snafu(b"2"), 2);
        assert_eq!(parse_snafu(b"1="), 3);
        assert_eq!(parse_snafu(b"1-"), 4);
        assert_eq!(parse_snafu(b"10"), 5);
        assert_eq!(parse_snafu(b"11"), 6);
        assert_eq!(parse_snafu(b"12"), 7);
        assert_eq!(parse_snafu(b"2="), 8);
        assert_eq!(parse_snafu(b"2-"), 9);
        assert_eq!(parse_snafu(b"1=0"), 15);
        assert_eq!(parse_snafu(b"1-0"), 20);
        assert_eq!(parse_snafu(b"1=11-2"), 2022);
        assert_eq!(parse_snafu(b"1-0---0"), 12345);
        assert_eq!(parse_snafu(b"1121-1110-1=0"), 314159265);

        assert_eq!(parse_snafu(b"10="), 23);
        assert_eq!(parse_snafu(b"10-"), 24);
        assert_eq!(parse_snafu(b"100"), 25);
        assert_eq!(parse_snafu(b"1000"), 125);

        assert_eq!(parse_snafu(b"12--1=10200-22"), 1651646862);
    }
}