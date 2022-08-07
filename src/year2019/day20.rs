use common::geo::Point;
use common::graph::Graph;
use common::grid2::{FixedGrid, GetterGrid, IterableSliceGrid};
use common::grid2::VecGrid;
use common::search::{astar, AStarState, BFS, BFSExploreState, BFSState};

common::day!(parse, part1, part2, 100, 1000, 1000);

fn part1(graph: &Graph<u16, (u32, i32)>) -> u32 {
    let aa = graph.find(&AA).unwrap();
    let zz = graph.find(&ZZ).unwrap();
    let init = GraphPathState { index: aa };

    astar(graph,
          &zz,
          init, 0u32,
          true,
          false,
    ).unwrap().0
}

fn part2(graph: &Graph<u16, (u32, i32)>) -> u32 {
    let aa = graph.find(&AA).unwrap();
    let zz = graph.find(&ZZ).unwrap();
    let init = GraphPathStateP2 { level: 0, index: aa };

    astar(graph,
          &zz,
          init, 0u32,
          true,
          false,
    ).unwrap().0
}

fn parse(data: &[u8]) -> Graph<u16, (u32, i32)> {
    let raw_grid = VecGrid::parse_lines(data, b'\n').unwrap();
    let mut data = Vec::with_capacity(raw_grid.width() * raw_grid.height());
    let mut graph = Graph::new();
    let mut positions = Vec::with_capacity(32);

    for (Point(x, y), v) in raw_grid.cells() {
        if *v != b'.' {
            data.push(MazePosition::Wall);
            continue;
        }

        let outer = x == 2 || y == 2 || x == raw_grid.width() - 3 || y == raw_grid.height() - 3;
        let u1 = raw_grid.get(&Point(x, y - 1)).unwrap();
        let u2 = raw_grid.get(&Point(x, y - 2)).unwrap();
        if let Some(portal_id) = portal_id(outer, [*u2, *u1]) {
            data.push(MazePosition::Portal(portal_id));
            graph.insert(portal_id);
            positions.push(Point(x, y));
            continue;
        }
        let d1 = raw_grid.get(&Point(x, y + 1)).unwrap();
        let d2 = raw_grid.get(&Point(x, y + 2)).unwrap();
        if let Some(portal_id) = portal_id(outer, [*d1, *d2]) {
            data.push(MazePosition::Portal(portal_id));
            graph.insert(portal_id);
            positions.push(Point(x, y));
            continue;
        }
        let l1 = raw_grid.get(&Point(x - 1, y)).unwrap();
        let l2 = raw_grid.get(&Point(x - 2, y)).unwrap();
        if let Some(portal_id) = portal_id(outer, [*l2, *l1]) {
            data.push(MazePosition::Portal(portal_id));
            graph.insert(portal_id);
            positions.push(Point(x, y));
            continue;
        }
        let r1 = raw_grid.get(&Point(x + 1, y)).unwrap();
        let r2 = raw_grid.get(&Point(x + 2, y)).unwrap();
        if let Some(portal_id) = portal_id(outer, [*r1, *r2]) {
            data.push(MazePosition::Portal(portal_id));
            graph.insert(portal_id);
            positions.push(Point(x, y));
            continue;
        }

        data.push(MazePosition::Ground);
    }

    let grid = VecGrid::new_from(raw_grid.width(), data);

    let mut bfs = BFS::new();
    for i in 0..graph.len() {
        let my_id = *graph.node(i).unwrap();
        let pos = positions[i];

        bfs.explore(&grid, GridBFSState { pos });
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

#[derive(Hash, Eq, PartialEq, Clone)]
struct GraphPathState {
    index: usize,
}

impl AStarState<Graph<u16, (u32, i32)>, usize, u32> for GraphPathState {
    fn heuristic(&self, _graph: &Graph<u16, (u32, i32)>, _goal: &usize) -> u32 {
        0
    }

    fn is_goal(&self, _graph: &Graph<u16, (u32, i32)>, goal: &usize) -> bool {
        self.index == *goal
    }

    fn next(&self, graph: &Graph<u16, (u32, i32)>, buffer: &mut Vec<(u32, Self)>) {
        for (new_index, (dist, _)) in graph.edges(self.index).unwrap() {
            buffer.push((*dist, Self { index: *new_index }));
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct GraphPathStateP2 {
    index: usize,
    level: i32,
}

impl AStarState<Graph<u16, (u32, i32)>, usize, u32> for GraphPathStateP2 {
    fn heuristic(&self, _graph: &Graph<u16, (u32, i32)>, _goal: &usize) -> u32 {
        0
    }

    fn is_goal(&self, _graph: &Graph<u16, (u32, i32)>, goal: &usize) -> bool {
        self.index == *goal
    }

    fn next(&self, graph: &Graph<u16, (u32, i32)>, buffer: &mut Vec<(u32, Self)>) {
        for (new_index, (dist, level_change)) in graph.edges(self.index).unwrap() {
            let portal_id = graph.node(*new_index).unwrap();

            if self.level == 0 {
                // Skip if it's going from to an outer (aside from warping inward)
                if *portal_id >= OUTER && *portal_id != ZZ && *level_change == 0 {
                    continue
                }

                buffer.push((*dist, Self { level: self.level + *level_change, index: *new_index }));
            } else {
                // AA and ZZ are walls on the inner levels.
                if *portal_id == AA || *portal_id == ZZ {
                    continue;
                }

                buffer.push((*dist, Self { level: self.level + *level_change, index: *new_index }));
            }
        }
    }
}


#[derive(Hash, Eq, PartialEq, Clone)]
struct GridBFSState {
    pos: Point<usize>,
}

impl GridBFSState {
    fn try_direction<G>(&self, maze: &G, new_pos: Point<usize>, buffer: &mut Vec<Self>) where G: GetterGrid<MazePosition> {
        if let Some(ch) = maze.get(&new_pos) {
            if *ch != MazePosition::Wall {
                buffer.push(Self { pos: new_pos });
            }
        }
    }
}

impl<B> BFSState<B, u16> for GridBFSState where B: GetterGrid<MazePosition> {
    fn is_goal(&self, board: &B, goal: &u16) -> bool {
        if let Some(mp) = board.get(&self.pos) {
            mp.eq(&MazePosition::Portal(*goal))
        } else {
            false
        }
    }

    fn next(&self, board: &B, buffer: &mut Vec<Self>) {
        let Point(x, y) = self.pos;

        self.try_direction(board, Point(x, y - 1), buffer);
        self.try_direction(board, Point(x - 1, y), buffer);
        self.try_direction(board, Point(x + 1, y), buffer);
        self.try_direction(board, Point(x, y + 1), buffer);
    }
}

impl<B> BFSExploreState<B, u16> for GridBFSState where B: GetterGrid<MazePosition> {
    fn has_goal(&self, board: &B) -> Option<u16> {
        if let Some(mp) = board.get(&self.pos) {
            match mp {
                MazePosition::Portal(portal_id) => Some(*portal_id),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[derive(Eq, PartialEq)]
enum MazePosition {
    Portal(u16),
    Ground,
    Wall,
}

#[allow(unused)]
fn format_pid(portal_id: u16) -> String {
    let l1 = (portal_id / 26) % 26;
    let l2 = portal_id % 26;

    if portal_id >= OUTER {
        format!("Outer {}{}", (l1 as u8 + b'A') as char, (l2 as u8 + b'A') as char)
    } else {
        format!("Inner {}{}", (l1 as u8 + b'A') as char, (l2 as u8 + b'A') as char)
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
