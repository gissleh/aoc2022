use common::geo::Point;
use common::graph::Graph;
use common::grid2::{FixedGrid, GetterMutGrid, IterableSliceGrid};
use common::grid2::VecGrid;
use common::search::{BFS, BFSResult, Dijkstra, DijkstraResult};

common::day!(parse, part1, part2, 100, 1000, 50);

fn part1(graph: &Graph<u16, (u32, i32)>) -> u32 {
    let aa = graph.find(&AA).unwrap();
    let zz = graph.find(&ZZ).unwrap();

    let mut dijkstra = Dijkstra::new(true, true);
    dijkstra.run(aa, 0u32, |index| {
        if *index == zz {
            DijkstraResult::Success
        } else {
            DijkstraResult::Continue(graph.edges(*index).unwrap().map(|(next_index, (move_cost, _))| {
                (*move_cost, 0, *next_index)
            }).collect())
        }
    }).unwrap().0
}

fn part2(graph: &Graph<u16, (u32, i32)>) -> u32 {
    let aa = graph.find(&AA).unwrap();
    let zz = graph.find(&ZZ).unwrap();

    let mut dijkstra = Dijkstra::new(true, false);
    dijkstra.run((aa, 0), 0u32, |(index, level)| {
        if *index == zz {
            DijkstraResult::Success
        } else {
            DijkstraResult::Continue(graph.edges(*index).unwrap().filter_map(|(next_index, (move_cost, level_diff))| {
                let portal_id = graph.node(*next_index).unwrap();

                if *level == 0 {
                    // Skip if it's going from to an outer (aside from warping inward)
                    if *portal_id >= OUTER && *portal_id != ZZ && *level_diff == 0 {
                        return None;
                    }
                } else {
                    // AA and ZZ are walls on the inner levels.
                    if *portal_id == AA || *portal_id == ZZ {
                        return None;
                    }
                }

                Some((*move_cost, 0, (*next_index, *level + level_diff)))
            }).collect())
        }
    }).unwrap().0
}

fn parse(data: &[u8]) -> Graph<u16, (u32, i32)> {
    let raw_grid = VecGrid::parse_lines(data, b'\n').unwrap();
    let mut data = Vec::with_capacity(raw_grid.width() * raw_grid.height());
    let mut graph = Graph::new();
    let mut positions = Vec::with_capacity(32);

    for (Point(x, y), v) in raw_grid.cells() {
        match *v {
            b'.' => { data.push(MazeCell::Ground); }
            b'#' | b' ' => { data.push(MazeCell::Wall); }
            b'A'..=b'Z' => {
                let mut v2 = 0u8;
                let mut pos: Point<usize> = Point(0, 0);

                let r = raw_grid.get(&Point(x + 1, y)).unwrap_or(&b' ');
                if *r >= b'A' && *r <= b'Z' {
                    v2 = *r;

                    pos = Point(x + 2, y);
                    if raw_grid.get(&pos) != Some(&b'.') {
                        pos = Point(x - 1, y);
                    }
                }
                let d = raw_grid.get(&Point(x, y + 1)).unwrap_or(&b' ');
                if *d >= b'A' && *d <= b'Z' {
                    v2 = *d;

                    pos = Point(x, y + 2);
                    if raw_grid.get(&pos) != Some(&b'.') {
                        pos = Point(x, y - 1);
                    }
                }

                if pos != Point(0, 0) {
                    let is_outer = pos.0 == 2 || pos.1 == 2 || pos.0 == raw_grid.width() - 3 || pos.1 == raw_grid.height() - 3;
                    let portal_id = portal_id(is_outer, [*v, v2]).unwrap();

                    positions.push(pos);
                    graph.insert(portal_id);
                }

                data.push(MazeCell::Wall);
            }
            _ => unreachable!()
        }
    }

    let mut grid = VecGrid::new_from(raw_grid.width(), data);
    for i in 0..graph.len() {
        let portal_id = *graph.node(i).unwrap();

        *grid.get_mut(&positions[i]).unwrap() = MazeCell::Portal(portal_id);
    }

    #[cfg(test)]
    grid.print(|v| {
        match *v {
            MazeCell::Wall => ('#', None),
            MazeCell::Ground => ('.', None),
            MazeCell::Portal(pid) => (':', Some(format_pid(pid))),
        }
    });

    let mut bfs = BFS::new();
    for i in 0..graph.len() {
        let my_id = *graph.node(i).unwrap();
        let pos = positions[i];

        bfs.run(pos, |p| {
            let valid_neighbours = p.cardinals().into_iter().filter(|p| {
                if let Some(c) = grid.get(p) {
                    *c != MazeCell::Wall
                } else {
                    false
                }
            }).collect();

            match grid.get(p).unwrap() {
                MazeCell::Wall => BFSResult::DeadEnd,
                MazeCell::Ground => BFSResult::Continue(valid_neighbours),
                MazeCell::Portal(pid) => BFSResult::Found(*pid, valid_neighbours),
            }
        });

        for (other_id, steps) in bfs.found_goals().iter() {
            if *other_id == my_id {
                continue;
            }

            let j = graph.find(other_id).unwrap();
            graph.connect(i, j, (*steps, 0));
        }

        if my_id >= OUTER {
            if let Some(j) = graph.find(&(my_id - OUTER)) {
                graph.connect(i, j, (1, 1));
                graph.connect(j, i, (1, -1));
            }
        }
    }

    graph
}

#[derive(Eq, PartialEq)]
enum MazeCell {
    Portal(u16),
    Ground,
    Wall,
}

#[allow(unused)]
fn format_pid(portal_id: u16) -> String {
    let l1 = (portal_id / 26) % 26;
    let l2 = portal_id % 26;

    if portal_id >= OUTER {
        format!("O{}{}", (l1 as u8 + b'A') as char, (l2 as u8 + b'A') as char)
    } else {
        format!("I{}{}", (l1 as u8 + b'A') as char, (l2 as u8 + b'A') as char)
    }
}

fn portal_id(outer: bool, name: [u8; 2]) -> Option<u16> {
    if name[0] < b'A' || name[0] > b'Z' {
        return None;
    }
    if name[1] < b'A' || name[1] > b'Z' {
        return None;
    }

    let name = [name[0] - b'A', name[1] - b'A'];
    Some(if outer {
        OUTER + (name[0] as u16 * 26) + name[1] as u16
    } else {
        (name[0] as u16 * 26) + name[1] as u16
    })
}

const OUTER: u16 = 26 * 26;
const AA: u16 = OUTER + 0;
const ZZ: u16 = OUTER + ((26 * 26) - 1);
