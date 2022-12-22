use common::aoc::Day;
use common::constants::{U32_3WINDOWS, U32_BITS};
use common::geo::{Point, Vertex};
use common::grid2::{ArrayGrid, FixedGrid, GetterMutGrid};
use common::parse3::{Parser, unsigned_int};
use common::search2;
use common::search2::{Bounded, Search};

pub fn main(day: &mut Day, input: &[u8]) {
    let (grid, max) = day.run_parse(1000, || parse(input));

    day.note("Max", max);

    day.run(1, "", 10000, || part1(&grid, &max));
    day.run(2, "BFS", 200, || part2_dfs(&grid, &max));
    day.run(2, "DFS", 200, || part2_bfs(&grid, &max));
}

fn parse(data: &[u8]) -> (ArrayGrid<u32, 1024, 32>, Vertex<usize>) {
    let parser = Vertex::comma_separated_parser(unsigned_int::<usize>()).skip(b'\n');

    let mut grid: ArrayGrid<u32, 1024, 32> = ArrayGrid::new();
    let mut max = Vertex(0, 0, 0);
    for v in parser.iterate(data) {
        let v = v + Vertex(2, 2, 2);
        if v.0 > max.0 { max.0 = v.0; }
        if v.1 > max.1 { max.1 = v.1; }
        if v.2 > max.2 { max.2 = v.2; }
        *grid.get_mut(&Point(v.0, v.1)).unwrap() |= U32_BITS[v.2];
    }

    (grid, max)
}

fn part1<G: GetterMutGrid<u32> + FixedGrid>(grid: &G, max: &Vertex<usize>) -> u32 {
    let mut surface_area = 0;
    for y in 1..=max.1 {
        for x in 1..=max.0 {
            let p = Point(x, y);
            let current = *grid.get(&p).unwrap();
            let current_ones = current.count_ones();

            // Add surface area from x,y neighbors
            surface_area += p.cardinals().iter()
                .map(|p2| *grid.get(&p2).unwrap())
                .map(|neighbor| current_ones - (current & neighbor).count_ones())
                .sum::<u32>();

            // Add surface area from z points
            for z in 2..=max.2 {
                if current & U32_BITS[z] != 0 {
                    surface_area += 3 - (current & U32_3WINDOWS[z]).count_ones();
                }
            }
        }
    }

    surface_area
}

fn part2<G: GetterMutGrid<u32>, S: Search<Vertex<usize>>>(grid: &G, max: &Vertex<usize>, search: S) -> usize {
    let m = *max + Vertex(1, 1, 1);

    search
        .bounded(|s| s.0 > 0 && s.1 > 0 && s.2 > 0 && s.0 <= m.0 && s.1 <= m.1 && s.2 <= m.2)
        .run(|dfs: &mut Bounded<S, _>, s: &Vertex<usize>| {
            match grid.get(&Point(s.0, s.1)) {
                Some(v) => {
                    if *v & U32_BITS[s.2] != 0 {
                        None
                    } else {
                        let cardinals = s.cardinals_offset(1);

                        let count = cardinals.iter()
                            .map(|p| *grid.get(&Point(p.0, p.1)).unwrap_or(&0) & U32_BITS[p.2])
                            .filter(|v| *v != 0)
                            .count();

                        for p in cardinals.iter() {
                            dfs.add_step(*p);
                        }

                        if count > 0 {
                            Some(count)
                        } else {
                            None
                        }
                    }
                }
                None => None,
            }
        })
        .sum()
}

fn part2_dfs<G: GetterMutGrid<u32>>(grid: &G, max: &Vertex<usize>) -> usize {
    part2(grid, max, search2::dfs(Vertex(1, 1, 1)))
}

fn part2_bfs<G: GetterMutGrid<u32>>(grid: &G, max: &Vertex<usize>) -> usize {
    part2(grid, max, search2::bfs(Vertex(1, 1, 1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_SIMPLE_EXAMPLE: &[u8] = b"3,3,3\n4,3,3\n";
    const P1_EXAMPLE: &[u8] = include_bytes!("./test_fixtures/d18_p1_example.txt");

    #[test]
    fn p2_works_on_example() {
        let (grid_simple, max_simple) = parse(P1_SIMPLE_EXAMPLE);
        let (grid, max) = parse(P1_EXAMPLE);

        assert_eq!(part2_dfs(&grid_simple, &max_simple), 10);
        assert_eq!(part2_dfs(&grid, &max), 58);
        assert_eq!(part2_bfs(&grid_simple, &max_simple), 10);
        assert_eq!(part2_dfs(&grid, &max), 58);
    }
}