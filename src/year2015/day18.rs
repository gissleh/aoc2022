use std::mem;
use common::geo::Point;
use common::grid2::{ArrayGrid, CountableGrid, FixedGrid, GetterMutGrid, NeighborCountGrid};
common::day!(parse, part1, part2, 1000, 50, 50);

fn part1<G>(grid: &G) -> u32 where G: FixedGrid + GetterMutGrid<u8> + Clone + NeighborCountGrid<u8> + CountableGrid<u8> {
    part1_with_steps::<G, 100>(grid)
}

fn part1_with_steps<G, const N: usize>(grid: &G) -> u32 where G: FixedGrid + GetterMutGrid<u8> + Clone + NeighborCountGrid<u8> + CountableGrid<u8> {
    let mut curr_grid = grid.clone();
    let mut next_grid = grid.clone();

    for _ in 0..N {
        for pos in Point::range(grid.width(), grid.height()) {
            let neigh_count = curr_grid.count_neighbors(&pos, &b'#');
            let curr = curr_grid.get(&pos).unwrap();
            let next = next_grid.get_mut(&pos).unwrap();
            conveys_step(neigh_count, curr, next);
        }

        mem::swap(&mut curr_grid, &mut next_grid);
    }

    curr_grid.count_occurrences_of(&b'#') as u32
}

fn part2<G>(grid: &G) -> u32 where G: FixedGrid + GetterMutGrid<u8> + Clone + NeighborCountGrid<u8> + CountableGrid<u8> {
    part2_with_steps::<G, 100>(grid)
}

fn part2_with_steps<G, const N: usize>(grid: &G) -> u32 where G: FixedGrid + GetterMutGrid<u8> + Clone + NeighborCountGrid<u8> + CountableGrid<u8> {
    let mut curr_grid = grid.clone();
    let mut next_grid = grid.clone();

    let cx = grid.width() - 1;
    let cy = grid.height() - 1;

    for _ in 0..N {
        for pos in Point::range(grid.width(), grid.height()) {
            if (pos.0 == 0 || pos.0 == cx) && (pos.1 == 0 || pos.1 == cy) {
                continue
            }

            let neigh_count = curr_grid.count_neighbors(&pos, &b'#');
            let curr = curr_grid.get(&pos).unwrap();
            let next = next_grid.get_mut(&pos).unwrap();
            conveys_step(neigh_count, curr, next);
        }

        mem::swap(&mut curr_grid, &mut next_grid);
    }

    curr_grid.count_occurrences_of(&b'#') as u32
}

fn conveys_step(neigh_count: usize, curr: &u8, next: &mut u8) {
    if *curr == b'#' {
        if neigh_count != 2 && neigh_count != 3 {
            *next = b'.'
        } else {
            *next = b'#'
        }
    } else {
        if neigh_count == 3 {
            *next = b'#'
        } else {
            *next = b'.'
        }
    }
}

fn parse(input: &[u8]) -> ArrayGrid<u8, 10000, 100> {
    parse2(input)
}

fn parse2<const W: usize, const S: usize>(input: &[u8]) -> ArrayGrid<u8, S, W> {
    let mut data = [0u8; S];
    let mut i = 0;

    for ch in input {
        match *ch {
            b'.' => {
                data[i] = *ch;
                i += 1;
            }
            b'#' => {
                data[i] = *ch;
                i += 1;
            }
            _ => {}
        }
    }

    return ArrayGrid::from_array(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &'static [u8] = b".#.#.#
...##.
#....#
..#...
#.#..#
####..";

    const P2_EXAMPLE: &'static [u8] = b"##.#.#
...##.
#....#
..#...
#.#..#
####.#";

    #[test]
    fn test_part1() {
        let grid = parse2::<6, 36>(P1_EXAMPLE);
        assert_eq!(part1_with_steps::<_, 5>(&grid), 4)
    }

    #[test]
    fn test_part2() {
        let grid = parse2::<6, 36>(P2_EXAMPLE);
        assert_eq!(part2_with_steps::<_, 5>(&grid), 17)
    }
}