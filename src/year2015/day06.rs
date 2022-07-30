use common::geo::Point;
use common::grid::ArrayGrid;
use common::parse;

common::day!(parse, part1, part2, 10000, 100, 50);

fn parse(data: &[u8]) -> Vec<Line> {
    let mut lines: Vec<Line> = Vec::with_capacity(data.len() / 23);

    let mut input = data;
    while let Some((line, new_input)) = Line::parse(input) {
        lines.push(line);
        input = new_input;
    }

    lines
}

fn part1(input: &[Line]) -> usize {
    let mut grid: ArrayGrid<bool, 1000000> = ArrayGrid::new_arr(1000, false);

    for Line{op, p1, p2} in input.iter() {
        match op {
            Operation::TurnOn => {
                grid.fill_rect(*p1, *p2, true);
            }
            Operation::TurnOff => {
                grid.fill_rect(*p1, *p2, false);
            }
            Operation::Toggle => {
                grid.map_rect(*p1, *p2, |v, _| {
                    *v = !*v;
                });
            }
        }
    }

    grid.iter().filter(|(_, v)| **v).count()
}

fn part2(input: &[Line]) -> u64 {
    let mut grid: ArrayGrid<u64, 1000000> = ArrayGrid::new_arr(1000, 0);

    for Line{op, p1, p2} in input.iter() {
        match op {
            Operation::TurnOn => {
                grid.map_rect(*p1, *p2, |v, _| {
                    *v += 1;
                });
            }
            Operation::TurnOff => {
                grid.map_rect(*p1, *p2, |v, _| {
                    if *v > 0 {
                        *v -= 1;
                    }
                });
            }
            Operation::Toggle => {
                grid.map_rect(*p1, *p2, |v, _| {
                    *v += 2;
                });
            }
        }
    }

    grid.iter().map(|(_, v)| *v).sum()
}

#[derive(Eq, PartialEq, Debug)]
enum Operation {
    TurnOn,
    TurnOff,
    Toggle,
}

#[derive(Debug)]
struct Line {
    op: Operation,
    p1: Point<usize>,
    p2: Point<usize>,
}

impl Line {
    fn parse(input: &[u8]) -> Option<(Line, &[u8])> {
        let mut input = input;
        let op = if let Some((_, new_input)) = parse::expect_bytes(b"turn on ")(input) {
            input = new_input;
            Operation::TurnOn
        } else if let Some((_, new_input)) = parse::expect_bytes(b"turn off ")(input) {
            input = new_input;
            Operation::TurnOff
        } else if let Some((_, new_input)) = parse::expect_bytes(b"toggle ")(input) {
            input = new_input;
            Operation::Toggle
        } else {
            return None;
        };

        let (x1, input) = parse::uint::<usize>(input)?;
        let (_, input) = parse::expect_byte::<b','>(input)?;
        let (y1, input) = parse::uint::<usize>(input)?;
        let (_, input) = parse::expect_bytes(b" through ")(input)?;
        let (x2, input) = parse::uint::<usize>(input)?;
        let (_, input) = parse::expect_byte::<b','>(input)?;
        let (y2, input) = parse::uint::<usize>(input)?;
        let (_, input) = parse::line(input)?;

        Some((
            Line {
                op,
                p1: Point(x1, y1),
                p2: Point(x2+1, y2+1),
            },
            input,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        let (line, _) = Line::parse(b"turn on 0,0 through 999,999").unwrap();
        assert_eq!(line.op, Operation::TurnOn);
        assert_eq!(line.p1, Point(0usize, 0usize));
        assert_eq!(line.p2, Point(1000usize, 1000usize));
        let (line, _) = Line::parse(b"toggle 0,0 through 999,0").unwrap();
        assert_eq!(line.op, Operation::Toggle);
        assert_eq!(line.p1, Point(0usize, 0usize));
        assert_eq!(line.p2, Point(1000usize, 1usize));
        let (line, _) = Line::parse(b"turn off 499,499 through 500,500").unwrap();
        assert_eq!(line.op, Operation::TurnOff);
        assert_eq!(line.p1, Point(499, 499));
        assert_eq!(line.p2, Point(501,501));
    }

    #[test]
    fn p1_turn_on_all() {
        let (line, _) = Line::parse(b"turn on 0,0 through 999,999").unwrap();
        assert_eq!(part1(&[line]), 1000000);
    }

    #[test]
    fn p1_turn_off_middle() {
        let (line1, _) = Line::parse(b"turn on 0,0 through 999,999").unwrap();
        let (line2, _) = Line::parse(b"turn off 499,499 through 500,500").unwrap();
        assert_eq!(part1(&[line1, line2]), 999996);
    }

    #[test]
    fn p1_toggle_rows() {
        let (line1, _) = Line::parse(b"turn on 0,0 through 999,999").unwrap();
        let (line2, _) = Line::parse(b"turn off 500,0 through 999,0").unwrap();
        let (line3, _) = Line::parse(b"toggle 0,0 through 999,0").unwrap();
        assert_eq!(part1(&[line1, line2, line3]), 999500);
    }
}