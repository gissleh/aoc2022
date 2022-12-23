use common::aoc::{Day, ResultPair};
use common::geo::{Point, Rect};
use common::grid2::{FixedGrid, GetterGrid, VecGrid};
use hashbrown::{HashMap, HashSet};

const MOVE_RULES: &[([Point<i32>; 3], Point<i32>); 4] = &[
    ([Point(-1, -1), Point(0, -1), Point(1, -1)], Point(0, -1)),
    ([Point(-1, 1), Point(0, 1), Point(1, 1)], Point(0, 1)),
    ([Point(-1, -1), Point(-1, 0), Point(-1, 1)], Point(-1, 0)),
    ([Point(1, -1), Point(1, 0), Point(1, 1)], Point(1, 0)),
];

pub fn main(day: &mut Day, input: &[u8]) {
    let initial_grid = day.run_parse(1000, || parse(input));

    day.run(3, "", 20, || puzzle(&initial_grid));
}

fn parse(data: &[u8]) -> VecGrid<u8> {
    VecGrid::new_from(
        data.iter().position(|v| *v == b'\n').unwrap(),
        data.iter().filter(|v| **v != b'\n').copied().collect(),
    )
}


fn puzzle<G>(initial_grid: &G) -> ResultPair<i32, i32> where G: GetterGrid<u8> + FixedGrid {
    let mut elves = HashSet::<Point<i32>>::with_capacity(1024);
    let mut next_moves = HashMap::<Point<i32>, Point<i32>>::with_capacity(1024);
    let mut bad_moves = HashSet::<Point<i32>>::with_capacity(1024);
    let mut result_after_10 = 0;
    let mut rounds = 1;

    for p in Point::range(initial_grid.width(), initial_grid.height()) {
        if *initial_grid.get(&p).unwrap() == b'#' {
            elves.insert(Point(p.0 as i32, p.1 as i32));
        }
    }

    let mut start_dir = 0;
    for n in 0.. {
        let mut any_moved = false;
        next_moves.clear();
        bad_moves.clear();

        for elf in elves.iter() {
            let found_any = elf.neighbors().iter()
                .find(|neigh| elves.contains(*neigh))
                .is_some();
            if !found_any {
                continue;
            }

            any_moved = true;

            for i in start_dir..start_dir + 4 {
                let (points, dir) = &MOVE_RULES[i % 4];

                if points.iter().map(|p| *p + *elf).find(|p| elves.contains(p)).is_none() {
                    let next_move = *elf + *dir;
                    if next_moves.insert(next_move, *elf).is_some() {
                        bad_moves.insert(next_move);
                    }

                    break;
                }
            }
        }

        #[cfg(test)] println!("Round {}", n + 1);
        for (dst, src) in next_moves.iter() {
            if bad_moves.contains(dst) {
                continue;
            }

            #[cfg(test)] println!("{:?} -> {:?}", src, dst);
            elves.remove(src);
            elves.insert(*dst);
        }

        if n == 9 {
            let elf = elves.iter().next().unwrap();
            let mut rect = Rect(*elf, *elf);
            for elf in elves.iter() {
                rect.envelop(elf)
            }

            result_after_10 = rect.area() - (elves.len() as i32);

            #[cfg(test)] println!("{:?}", elves);
            #[cfg(test)] println!("{:?} {}", rect, result_after_10);
        }

        if n >= 10 && !any_moved {
            break;
        }

        start_dir += 1;
        rounds += 1;
    }

    ResultPair(result_after_10, rounds)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE_1: &[u8] = include_bytes!("./test_fixtures/d23_p1_example_1.txt");
    const P1_EXAMPLE_2: &[u8] = include_bytes!("./test_fixtures/d23_p1_example_2.txt");

    #[test]
    fn p1_works_on_example() {
        let ResultPair(p1, _) = puzzle(&parse(P1_EXAMPLE_2));
        assert_eq!(p1, 25);

        let ResultPair(p1, _) = puzzle(&parse(P1_EXAMPLE_1));
        assert_eq!(p1, 110);
    }

    #[test]
    fn p2_works_on_example() {
        let ResultPair(_, p2) = puzzle(&parse(P1_EXAMPLE_1));
        assert_eq!(p2, 20);
    }
}
