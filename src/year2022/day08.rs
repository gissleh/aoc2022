use std::cmp::max;
use common::aoc::{Day, ResultPair};
use common::geo::Point;
use common::grid2::{FixedGrid, GetterMutGrid, VecGrid};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Grid Size", format!("{},{}", input.width(), input.height()));

    day.run(3, "", 100, || both_parts(&input));
}

fn parse(data: &[u8]) -> VecGrid<u8> {
    let width = data.iter().take_while(|b| **b != b'\n').count();

    VecGrid::new_from(width, data.iter()
        .filter(|b| **b != b'\n')
        .map(|b| *b - b'0')
        .collect(),
    )
}

fn both_parts<G>(input: &G) -> ResultPair<usize, u32> where G: GetterMutGrid<u8> + FixedGrid {
    let mut visible_count = 0;
    let mut best_scenic_score = 0;
    for y in 1..input.height() - 1 {
        for x in 1..input.width() - 1 {
            let current = Point(x, y);
            let current_value = input.get(&current).unwrap();

            let mut up = 0;
            let mut right = 0;
            let mut down = 0;
            let mut left = 0;
            let mut hidden = 0u8;

            for x in (0..x).rev() {
                left += 1;

                let neighbor_value = input.get(&Point(x, y)).unwrap();
                if *neighbor_value >= *current_value {
                    hidden |= 0b0001;
                    break;
                }
            }
            for x in x + 1..input.width() {
                right += 1;

                let neighbor_value = input.get(&Point(x, y)).unwrap();
                if *neighbor_value >= *current_value {
                    hidden |= 0b0010;
                    break;
                }
            }
            for y in (0..y).rev() {
                up += 1;

                let neighbor_value = input.get(&Point(x, y)).unwrap();
                if *neighbor_value >= *current_value {
                    hidden |= 0b0100;
                    break;
                }
            }
            for y in y + 1..input.height() {
                down += 1;

                let neighbor_value = input.get(&Point(x, y)).unwrap();
                if *neighbor_value >= *current_value {
                    hidden |= 0b1000;
                    break;
                }
            }

            if hidden != 0b1111 {
                #[cfg(test)] println!("Visible position {},{}", x, y);
                visible_count += 1;
            }

            #[cfg(test)] println!("Scenic score {},{}: {}", x, y, up * down * left * right);
            best_scenic_score = max(best_scenic_score, up * down * left * right);
        }
    }

    ResultPair(
        visible_count + (input.height() * 2) + (input.width() * 2) - 4,
        best_scenic_score,
    )
}

#[cfg(test)]
mod tests {
    use common::grid2::{RowGrid};
    use super::*;

    const EXAMPLE_DATA: &'static [u8] = b"30373
25512
65332
33549
35390
";

    #[test]
    fn parse_works_on_example() {
        assert_eq!(parse(EXAMPLE_DATA).row(0).unwrap(), &[3u8, 0, 3, 7, 3]);
        assert_eq!(parse(EXAMPLE_DATA).row(1).unwrap(), &[2u8, 5, 5, 1, 2]);
        assert_eq!(parse(EXAMPLE_DATA).row(2).unwrap(), &[6u8, 5, 3, 3, 2]);
        assert_eq!(parse(EXAMPLE_DATA).row(3).unwrap(), &[3u8, 3, 5, 4, 9]);
        assert_eq!(parse(EXAMPLE_DATA).row(4).unwrap(), &[3u8, 5, 3, 9, 0]);
    }

    #[test]
    fn parts_works_on_example() {
        assert_eq!(both_parts(&parse(EXAMPLE_DATA)), ResultPair(21usize, 8u32));
    }
}