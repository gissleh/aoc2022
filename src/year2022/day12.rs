use common::aoc::Day;
use common::geo::Point;
use common::grid2::{FixedGrid, GetterMutGrid, VecGrid};
use common::search::{BFS, BFSResult};

pub fn main(day: &mut Day, input: &[u8]) {
    let (input, start_point, end_point) = day.run_parse(1000, || parse(input));

    day.run(1, "", 1000, || part1(&input, &start_point, &end_point));
    day.run(2, "", 1000, || part2(&input, &end_point));
}

fn parse(data: &[u8]) -> (VecGrid<u8>, Point<usize>, Point<usize>) {
    let width = data.iter().take_while(|b| **b != b'\n').count();
    let mut filtered_data = Vec::<u8>::with_capacity(data.len());
    let mut start_point = Point(0, 0);
    let mut end_point = Point(0, 0);

    for v in data {
        match *v {
            b'a'..=b'z' => filtered_data.push(*v - b'a'),
            b'S' => {
                start_point = Point(filtered_data.len() % width, filtered_data.len() / width);
                filtered_data.push(0);
            }
            b'E' => {
                end_point = Point(filtered_data.len() % width, filtered_data.len() / width);
                filtered_data.push(26);
            }
            _ => {}
        }
    }

    (VecGrid::new_from(width, filtered_data), start_point, end_point)
}

fn part1<G: FixedGrid + GetterMutGrid<u8>>(input: &G, start_point: &Point<usize>, end_point: &Point<usize>) -> u32 {
    let mut bfs: BFS<Point<usize>, u8> = BFS::new();

    let (_, distance) = bfs.run(*start_point, |p| {
        if let Some(elevation) = input.get(p) {
            if p == end_point {
                return BFSResult::Success;
            } else {
                BFSResult::Continue(p.cardinals().iter().filter(|p| {
                    if let Some(next_elevation) = input.get(*p) {
                        *next_elevation <= *elevation + 1
                    } else {
                        false
                    }
                }).copied().collect())
            }
        } else {
            BFSResult::DeadEnd
        }
    }).unwrap();

    distance
}

fn part2<G: FixedGrid + GetterMutGrid<u8>>(input: &G, end_point: &Point<usize>) -> u32 {
    let mut bfs: BFS<Point<usize>, u8> = BFS::new();

    let (_, distance) = bfs.run(*end_point, |p| {
        if let Some(elevation) = input.get(p) {
            if *elevation == 0 {
                return BFSResult::Success;
            } else {
                BFSResult::Continue(p.cardinals().iter().filter(|p| {
                    if let Some(next_elevation) = input.get(*p) {
                        *elevation <= *next_elevation + 1
                    } else {
                        false
                    }
                }).copied().collect())
            }
        } else {
            BFSResult::DeadEnd
        }
    }).unwrap();

    distance
}
