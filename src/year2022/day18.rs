use smallvec::SmallVec;
use common::aoc::Day;
use common::constants::{U32_3WINDOWS, U32_BITS};
use common::geo::{Point, Vertex};
use common::grid2::{ArrayGrid, FixedGrid, GetterGrid};
use common::parse3::{Parser, unsigned_int};
use common::search::{BFS, BFSResult};

pub fn main(day: &mut Day, input: &[u8]) {
    let grid = day.run_parse(1000, || parse(input));

    day.run(1, "", 10000, || part1(&grid));
    day.run(2, "", 200, || part2(&grid));
}

fn parse(data: &[u8]) -> ArrayGrid<u32, 1024, 32> {
    let parser = Vertex::comma_separated_parser(unsigned_int::<usize>())
        .skip(b'\n');

    let mut grid: ArrayGrid<u32, 1024, 32> = ArrayGrid::new();
    for v in parser.iterate(data) {
        *grid.get_mut(&Point(v.0 + 2, v.1 + 2)).unwrap() |= U32_BITS[v.2 + 2];
    }

    grid
}

fn part1<G: GetterGrid<u32> + FixedGrid>(grid: &G) -> u32 {
    let mut surface_area = 0;
    for y in 1..grid.height() - 1 {
        for x in 1..grid.width() - 1 {
            let p = Point(x, y);
            let current = *grid.get(&p).unwrap();
            let current_ones = current.count_ones();

            // Add surface area from x,y neighbors
            surface_area += p.cardinals().iter()
                .map(|p2| *grid.get(&p2).unwrap())
                .map(|neighbor| current_ones - (current & neighbor).count_ones())
                .sum::<u32>();

            // Add surface area from z points
            for z in 0..32 {
                if current & U32_BITS[z] != 0 {
                    surface_area += 3 - (current & U32_3WINDOWS[z]).count_ones();
                }
            }
        }
    }

    surface_area
}

fn part2<G: GetterGrid<u32>>(grid: &G) -> u32 {
    let mut bfs: BFS<Vertex<usize>, u32> = BFS::new();
    bfs.run(Vertex(1, 1, 1), |s: &Vertex<usize>| {
        if s.0 == 0 || s.1 == 0 || s.2 == 0
            || s.0 == 32 || s.1 == 32 || s.2 == 31 {
            return BFSResult::DeadEnd;
        }

        match grid.get(&Point(s.0, s.1)) {
            Some(v) => {
                if *v & U32_BITS[s.2] != 0 {
                    BFSResult::DeadEnd
                } else {
                    let cardinals: SmallVec<[Vertex<usize>; 16]> = s.cardinals_offset(1)
                        .iter().copied().collect();

                    let count = cardinals.iter()
                        .map(|p| *grid.get(&Point(p.0, p.1)).unwrap_or(&0) & U32_BITS[p.2])
                        .filter(|v| *v != 0)
                        .count();

                    if count > 0 {
                        BFSResult::Found(count as u32, cardinals)
                    } else {
                        BFSResult::Continue(cardinals)
                    }
                }
            }
            None => BFSResult::DeadEnd,
        }
    });

    bfs.found_goals().iter().map(|(count, _)| count).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_SIMPLE_EXAMPLE: &[u8] = b"3,3,3\n4,3,3\n";
    const P1_EXAMPLE: &[u8] = include_bytes!("./test_fixtures/d18_p1_example.txt");

    #[test]
    fn p2_works_on_example() {
        assert_eq!(part2(&parse(P1_SIMPLE_EXAMPLE)), 10);
        assert_eq!(part2(&parse(P1_EXAMPLE)), 58);
    }
}