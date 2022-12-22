use std::cmp::{max, min};
use smallvec::smallvec;
use common::aoc::{Day, ResultAndCarry};
use common::geo::Point;
use common::grid2::{FixedGrid, GetterMutGrid, render_grid, VecGrid};
use common::parse3::{choice, Parser, point, unsigned_int};
use common::search::{BFS, BFSResult};

const BLANK: u8 = 0b00;
const WALL: u8 = 0b10;
const SAND: u8 = 0b11;

pub fn main(day: &mut Day, input: &[u8]) {
    let (input, source_x) = day.run_parse(1000, || parse(input));

    day.note("Source X", source_x);
    day.note("Grid width", input.width());
    day.note("Grid Height", input.height());

    let ResultAndCarry(part1_result, new_grid) = day.run(1, "", 1000, || part1(input.clone(), source_x));
    day.run(2, "", 1000, move || part2(new_grid.clone(), part1_result, source_x));
    day.run(2, "BFS", 1000, || part2_bfs(&input, source_x));
}

fn parse(data: &[u8]) -> (VecGrid<u8>, usize) {
    #[derive(Copy, Clone)]
    enum Path {
        Point(Point<usize>),
        Skip,
    }

    // Parse the path.
    let path = choice((
        point(unsigned_int::<usize>()).skip(b" -> ").map(Path::Point),
        b'\n'.map_to_value(Path::Skip)
    )).repeat().parse(data).unwrap();

    // Constrain the grid to avoid over-allocating, but account for part2 making a perfect pyramid.
    let max_y = path.iter().fold(0usize, |max_y, segment| {
        match segment {
            Path::Point(Point(_, y)) => max(*y, max_y),
            Path::Skip => max_y,
        }
    });
    let min_x = 500 - (max_y * 2);
    let max_x = 500 + (max_y * 2);

    // Make the grid.
    let mut grid = VecGrid::new((max_x - min_x) + 1, max_y + 2);
    let mut prev: Option<Point<usize>> = None;
    for segment in path.into_iter() {
        if let Path::Point(mut curr) = segment {
            curr.0 -= min_x;

            if let Some(prev) = prev {
                if prev.0 == curr.0 {
                    for y in min(prev.1, curr.1)..=max(prev.1, curr.1) {
                        *grid.get_mut(&Point(curr.0, y)).unwrap() = WALL;
                    }
                } else {
                    for x in min(prev.0, curr.0)..=max(prev.0, curr.0) {
                        *grid.get_mut(&Point(x, curr.1)).unwrap() = WALL;
                    }
                }
            }

            prev = Some(curr);
        } else {
            prev = None;
        }
    }

    (grid, 500 - min_x)
}

fn part1<G>(mut grid: G, source_x: usize) -> ResultAndCarry<u32, G> where G: GetterMutGrid<u8> + FixedGrid {
    let mut sand_count = 0;
    let height = grid.height() - 1;
    let source = Point(source_x, 0);

    loop {
        let (sand_grain, settled) = simulate(&mut grid, &source, height - 1);
        if !settled {
            break;
        }

        if let Some(pos) = grid.get_mut(&sand_grain) {
            *pos = SAND;
            sand_count += 1;
        }
    }

    ResultAndCarry(sand_count, grid)
}

fn part2<G>(mut grid: G, mut sand_count: u32, source_x: usize) -> u32 where G: GetterMutGrid<u8> + FixedGrid {
    let height = grid.height() - 1;
    let source = Point(source_x, 0);

    loop {
        let (sand_grain, _) = simulate(&mut grid, &source, height);

        if let Some(cell) = grid.get_mut(&sand_grain) {
            *cell = SAND;
            sand_count += 1;

            if sand_grain.1 == 0 {
                break;
            }
        } else {
            panic!("{:?} is outside", sand_grain);
        }
    }

    sand_count
}

#[inline]
fn simulate<G>(grid: &mut G, source: &Point<usize>, height: usize) -> (Point<usize>, bool) where G: GetterMutGrid<u8> {
    let mut sand_grain = *source;

    while sand_grain.1 < height {
        let mut next = Point(sand_grain.0, sand_grain.1 + 1);
        if *grid.get(&next).unwrap() == BLANK {
            sand_grain = next;
            continue;
        }
        next.0 -= 1;
        if *grid.get(&next).unwrap() == BLANK {
            sand_grain = next;
            continue;
        }
        next.0 += 2;
        if *grid.get(&next).unwrap() == BLANK {
            sand_grain = next;
            continue;
        }

        return (sand_grain, true);
    }

    (sand_grain, false)
}

fn part2_bfs<G>(grid: &G, source_x: usize) -> usize where G: GetterMutGrid<u8> + FixedGrid {
    let mut bfs: BFS<Point<usize>, ()> = BFS::new();

    bfs.run(Point(source_x, 0), |pos| {
        if pos.1 == grid.height() {
            return BFSResult::DeadEnd;
        }

        if *grid.get(pos).unwrap() == BLANK {
            BFSResult::Found((), smallvec![
                Point(pos.0, pos.1 + 1),
                Point(pos.0 - 1, pos.1 + 1),
                Point(pos.0 + 1, pos.1 + 1),
            ])
        } else {
            BFSResult::DeadEnd
        }
    });

    bfs.found_goals().len()
}

#[allow(dead_code)]
fn render_sand_grid<G>(grid: &G) -> String where G: GetterMutGrid<u8> + FixedGrid {
    render_grid::<G, _, _>(&grid, |c| match *c {
        WALL => ('#', None),
        SAND => ('o', None),
        _ => ('·', None),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = b"498,4 -> 498,6 -> 496,6\n503,4 -> 502,4 -> 502,9 -> 494,9\n";

    #[test]
    fn p1_works_on_example() {
        let (grid, source_x) = parse(&P1_EXAMPLE);
        assert_eq!(part1(grid, source_x).0, 24);
    }

    #[test]
    fn p2_works_on_example() {
        let (grid, source_x) = parse(&P1_EXAMPLE);
        let ResultAndCarry(sand_count, grid) = part1(grid, source_x);
        assert_eq!(part2(grid, sand_count, source_x), 93);
    }

    #[test]
    fn p2_bfs_works_on_example() {
        let (grid, source_x) = parse(&P1_EXAMPLE);
        assert_eq!(part2_bfs(&grid, source_x), 93);
    }
}