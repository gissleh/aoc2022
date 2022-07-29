use common::parse;

common::day!(parse, part1, part2, 10000, 10000, 10000);

fn parse(data: &[u8]) -> Vec<u8> {
    let (res, _) = parse::line(data).unwrap();
    Vec::from(res)
}

fn part1(input: &[u8]) -> i32 {
    input.iter().fold(0, |v, c| {
        match c {
            b'(' => v + 1,
            b')' => v - 1,
            _ => unreachable!(),
        }
    })
}

fn part2(input: &[u8]) -> usize {
    let mut current_floor = 0;
    for (i, c) in input.iter().enumerate() {
        match c {
            b'(' => { current_floor += 1 },
            b')' => { current_floor -= 1 },
            _ => unreachable!(),
        }

        if current_floor == -1 {
            return i + 1;
        }
    }

    0
}

#[test]
fn test_part1() {
    assert_eq!(part1(b"(())"), 0);
    assert_eq!(part1(b"()()"), 0);
    assert_eq!(part1(b"((("), 3);
    assert_eq!(part1(b"(()(()("), 3);
    assert_eq!(part1(b"())"), -1);
    assert_eq!(part1(b"))("), -1);
    assert_eq!(part1(b")))"), -3);
    assert_eq!(part1(b")())())"), -3);
}

#[test]
fn test_part2() {
    assert_eq!(part2(b")"), 1);
    assert_eq!(part2(b"()())"), 5);
}