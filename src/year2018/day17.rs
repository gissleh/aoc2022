use std::cmp::{max, min};
use common::grid2::{FixedGrid, GetterGrid, RowGrid, VecGrid};
use common::parse2;
use common::geo::Point;

common::day!(parse, part1, part2, 10000, 10000, 10000);

enum Instruction {
    XY1Y2(usize, usize, usize),
    YX1X2(usize, usize, usize),
}

#[cfg(test)] const CELL_SAND: u8 = 0b00u8;
const CELL_WATER_SETTLED: u8 = 0b101u8;
const CELL_WATER_FLOWING: u8 = 0b110u8;
const CELL_CLAY: u8 = 0b11u8;
const CELL_SOLID_MASK: u8 = 0b01u8;
const CELL_WATER_MASK: u8 = 0b100u8;

const DOWN: Point<usize> = Point(0, 1);

fn parse(data: &[u8]) -> (VecGrid<u8>, usize) {
    let instructions: Vec<Instruction> = parse2::map(data, |input| parse2::expect_byte::<b'x'>(input)
        .or(parse2::expect_byte::<b'y'>)
        .and_discard(parse2::expect_byte::<b'='>)
        .and(parse2::uint::<usize>)
        .and_discard(parse2::expect_bytes(b", "))
        .and_discard(parse2::n_bytes::<2>)
        .and(parse2::uint::<usize>)
        .and_discard(parse2::expect_bytes(b".."))
        .and(parse2::uint::<usize>)
        .and_discard(parse2::expect_byte::<b'\n'>)
        .map(|(((c, p1), p2), p3)| {
            if c == b'x' {
                Instruction::XY1Y2(p1, p2, p3)
            } else {
                Instruction::YX1X2(p1, p2, p3)
            }
        }),
    ).collect();

    let (_min_x, min_y, max_x, max_y) = instructions.iter().fold(
        (usize::MAX, usize::MAX, 0usize, 0usize),
        |(min_x, min_y, max_x, max_y), i| {
            match i {
                Instruction::YX1X2(y, x1, x2) => (
                    min(min_x, *x1), min(min_y, *y),
                    max(max_x, *x2), max(max_y, *y),
                ),
                Instruction::XY1Y2(x, y1, y2) => (
                    min(min_x, *x), min(min_y, *y1),
                    max(max_x, *x), max(max_y, *y2),
                ),
            }
        },
    );

    let mut grid = VecGrid::new(max_x + 1, max_y + 1);

    for instruction in instructions {
        match instruction {
            Instruction::YX1X2(y, x1, x2) => {
                for x in x1..=x2 {
                    *grid.get_mut(&Point(x, y)).unwrap() = CELL_CLAY;
                }
            }
            Instruction::XY1Y2(x, y1, y2) => {
                for y in y1..=y2 {
                    *grid.get_mut(&Point(x, y)).unwrap() = CELL_CLAY;
                }
            }
        }
    }

    (grid, min_y)
}

fn part1(input: &(VecGrid<u8>, usize)) -> usize {
    solve(&input.0, input.1, |v| v & CELL_WATER_MASK == CELL_WATER_MASK)
}

fn part2(input: &(VecGrid<u8>, usize)) -> usize {
    solve(&input.0, input.1, |v| v == CELL_WATER_SETTLED)
}

fn solve<F>(grid: &VecGrid<u8>, min_y: usize, cb: F) -> usize where F: Fn(u8) -> bool {
    let mut grid = grid.clone();
    let max_y = grid.height() - 1;

    let mut stack: Vec<Point<usize>> = Vec::with_capacity(64);
    stack.push(Point(500, 0));

    while let Some(p) = stack.pop() {
        #[cfg(test)] {
            for x in 494..507 {
                if p.0 == x {
                    print!("v");
                } else {
                    print!(" ");
                }
            }
            println!();
            for y in min_y..=max_y {
                for x in 494..507 {
                    print!("{}", match *grid.get(&Point(x, y)).unwrap() {
                        CELL_SAND => '.',
                        CELL_CLAY => '#',
                        CELL_WATER_FLOWING => '|',
                        CELL_WATER_SETTLED => '~',
                        _ => unreachable!(),
                    })
                }

                if y == p.1 {
                    print!(" <");
                }

                println!();
            }
        }

        if p.1 == max_y {
            continue;
        }

        // Flow down if possible
        if let Some(v) = grid.get_mut(&(p + DOWN)) {
            if *v & CELL_SOLID_MASK == 0 {
                *v = CELL_WATER_FLOWING;
                stack.push(p + DOWN);
                continue;
            }
        }

        // Trickle left
        let mut left_x = p.0;
        let mut left_solid = true;
        for x in (1..=p.0).rev() {
            if grid.get(&Point(x, p.1 + 1)).unwrap() & CELL_SOLID_MASK == 0 {
                left_x = x;
                left_solid = false;
                break;
            }
            if grid.get(&Point(x - 1, p.1)).unwrap() & CELL_SOLID_MASK != 0 {
                left_x = x;
                left_solid = true;
                break;
            }
        }

        // Trickle right
        let mut right_x = p.0;
        let mut right_solid = true;
        for x in p.0..grid.width() - 1 {
            if grid.get(&Point(x, p.1 + 1)).unwrap() & CELL_SOLID_MASK == 0 {
                right_x = x;
                right_solid = false;
                break;
            }
            if grid.get(&Point(x + 1, p.1)).unwrap() & CELL_SOLID_MASK != 0 {
                right_x = x;
                right_solid = true;
                break;
            }
        }

        if left_solid && right_solid {
            for x in left_x..=right_x {
                *grid.get_mut(&Point(x, p.1)).unwrap() = CELL_WATER_SETTLED;
            }

            stack.push(Point(p.0, p.1 - 1));
        } else {
            for x in left_x..=right_x {
                *grid.get_mut(&Point(x, p.1)).unwrap() = CELL_WATER_FLOWING;
            }

            if !left_solid {
                stack.push(Point(left_x, p.1))
            }
            if !right_solid {
                stack.push(Point(right_x, p.1))
            }
        }
    }

    let res = (min_y..=max_y)
        .map(|y| grid.row(y).unwrap())
        .map(|r| r.iter().filter(|v| cb(**v)).count())
        .sum();

    res
}

#[test]
fn test_part1() {
    let input = parse(b"x=495, y=2..7
y=7, x=495..501
x=501, y=3..7
x=498, y=2..4
x=506, y=1..2
x=498, y=10..13
x=504, y=10..13
y=13, x=498..504
");

    assert_eq!(part1(&input), 57);
}