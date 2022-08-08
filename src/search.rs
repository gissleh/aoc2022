use std::collections::VecDeque;
use std::hash::Hash;
use std::ops::Add;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::SmallVec;

pub enum DijkstraResult<C, S> where S: Hash + Eq + Clone, C: Ord + Copy + Add<Output=C> {
    /// Same as continue but with a blank list.
    DeadEnd,
    /// This state ends the dijkstra search
    Success,
    /// Continue with these steps. The second C is a heuristic
    Continue(SmallVec<[(C, C, S); 16]>),
}

pub struct Dijkstra<S, C> where S: Hash + Eq + Clone, C: Ord + Copy + Add<Output=C> {
    seen: FxHashMap<S, C>,
    unexplored: Vec<(C, C, S)>,
    cost_only_increases: bool,
    return_first_success: bool,
}

impl<S, C> Dijkstra<S, C> where S: Hash + Eq + Clone, C: Ord + Copy + Add<Output=C> {
    pub fn run<F>(&mut self, initial_state: S, initial_cost: C, step_function: F) -> Option<(C, S)>
        where F: Fn(&S) -> DijkstraResult<C, S> {
        self.seen.clear();
        self.unexplored.clear();

        let mut lowest: Option<(C, S)> = None;
        let mut next: (C, S) = (initial_cost, initial_state.clone());

        self.seen.insert(initial_state, initial_cost);

        loop {
            let (cost, state) = next;

            match step_function(&state) {
                DijkstraResult::DeadEnd => {}
                DijkstraResult::Success => {
                    if self.return_first_success {
                        return Some((cost, state));
                    } else {
                        if let Some((lowest_cost, lowest_state)) = &mut lowest {
                            if cost < *lowest_cost {
                                *lowest_cost = cost;
                                *lowest_state = state;
                            }
                        } else {
                            lowest = Some((cost, state));
                        }
                    }
                }
                DijkstraResult::Continue(next_steps) => {
                    for (step_cost, step_heuristic, next_state) in next_steps.into_iter() {
                        let next_cost = cost + step_cost;

                        if self.cost_only_increases {
                            if let Some((lowest_cost, _)) = &lowest {
                                if *lowest_cost <= next_cost {
                                    continue;
                                }
                            }
                        }

                        if let Some(existing_cost) = self.seen.get(&next_state) {
                            if *existing_cost <= next_cost {
                                continue;
                            }
                        }

                        self.unexplored.push((next_cost, step_heuristic, next_state.clone()));
                        self.seen.insert(next_state.clone(), next_cost);
                    }
                }
            }

            if self.unexplored.is_empty() {
                break;
            }

            let mut winner_index = 0usize;
            let mut lowest_cost = self.unexplored[0].0 + self.unexplored[0].1;

            for (i, (next_cost, next_heuristic, _)) in self.unexplored.iter().enumerate().skip(1) {
                let estimated_cost = *next_cost + *next_heuristic;
                if estimated_cost < lowest_cost {
                    winner_index = i;
                    lowest_cost = estimated_cost;
                }
            }

            let (next_cost, _, next_state) = self.unexplored.swap_remove(winner_index);
            next = (next_cost, next_state);
        }

        lowest
    }

    pub fn new(cost_only_increases: bool, return_first_success: bool) -> Self {
        Self {
            seen: FxHashMap::default(),
            unexplored: Vec::with_capacity(64),
            cost_only_increases,
            return_first_success,
        }
    }
}

pub enum BFSResult<S, G> {
    /// Same as continue with an empty vec.
    DeadEnd,
    /// Return the main search with this state.
    Success,
    /// Continue with these possible steps.
    Continue(SmallVec<[S; 16]>),
    /// Log this goal and continue.
    Found(G, SmallVec<[S; 16]>),
}

pub struct BFS<S, G> where S: Eq + Hash + Clone {
    queue: VecDeque<(S, u32)>,
    seen: FxHashSet<S>,
    goals: SmallVec<[(G, u32); 16]>,
}

impl<S, G> BFS<S, G> where S: Eq + Hash + Clone {
    pub fn run<F>(&mut self, initial_state: S, step: F) -> Option<(S, u32)> where F: Fn(&S) -> BFSResult<S, G> {
        self.seen.clear();
        self.queue.clear();
        self.goals.clear();

        self.seen.insert(initial_state.clone());
        self.queue.push_back((initial_state, 0));

        while let Some((current_state, current_steps)) = self.queue.pop_front() {
            match step(&current_state) {
                BFSResult::DeadEnd => {}
                BFSResult::Success => {
                    return Some((current_state, current_steps));
                }
                BFSResult::Found(value, next_steps) => {
                    self.goals.push((value, current_steps));
                    self.add_steps(next_steps.into_iter(), current_steps + 1);
                }
                BFSResult::Continue(next_steps) => {
                    self.add_steps(next_steps.into_iter(), current_steps + 1);
                }
            }
        }

        None
    }

    pub fn found_goals(&self) -> &[(G, u32)] {
        &self.goals
    }

    pub fn new() -> BFS<S, G> {
        BFS {
            goals: SmallVec::new(),
            queue: VecDeque::with_capacity(64),
            seen: FxHashSet::default(),
        }
    }

    fn add_steps<I>(&mut self, iter: I, new_step: u32) where I: IntoIterator<Item=S> {
        for new_state in iter {
            if self.seen.contains(&new_state) {
                continue;
            }

            self.seen.insert(new_state.clone());
            self.queue.push_back((new_state, new_step));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::geo::Point;
    use crate::grid2::{FixedGrid, GetterGrid, VecGrid};
    use super::*;

    const MAZE: &'static [u8] = b"#################
#.......#...#...#
#.#####.#.#.#.#.#
#.#...#.......#.#
#.###.#########.#
#.....#.....#...#
#.#########...#.#
#...........#.#.#
#################
";

    const MAZE_2: &'static [u8] = b"#################
#a......#...#.i.#
#.#####.#.#.#.#.#
#.#.y.#..z....#.#
#.#############.#
#.....#.t...#r..#
#.#########...#.#
#.........b.#.#x#
#################
";

    const KNIGHT_MOVES: &'static [Point<i32>] = &[
        Point(1, 2),
        Point(1, -2),
        Point(2, 1),
        Point(-2, 1),
        Point(-1, 2),
        Point(-1, -2),
        Point(2, -1),
        Point(-2, -1),
    ];

    #[test]
    fn chess_piece_2() {
        let mut d = Dijkstra::new(true, true);
        let goal = Point(4, 6);
        let res = d.run(Point(0i32, 0i32), 0, |p| {
            if p.eq(&goal) {
                DijkstraResult::Success
            } else {
                DijkstraResult::Continue(KNIGHT_MOVES.iter().map(|m| {
                    let p2 = *p + *m;
                    let h = p2.distance(&goal);

                    (1, h, p2)
                }).collect())
            }
        });

        assert_eq!(res, Some((4, goal)));
    }

    #[test]
    fn maze_walker_2() {
        let maze = VecGrid::parse_lines(MAZE, b'\n').unwrap();
        let mut d = Dijkstra::new(true, true);
        let goal = Point(maze.width() - 2, maze.height() - 2);

        let res = d.run(Point(1usize, 1usize), 0, |p| {
            if *p == goal {
                DijkstraResult::Success
            } else {
                DijkstraResult::Continue(p.cardinals().into_iter().filter_map(|p2| {
                    if let Some(v) = maze.get(&p2) {
                        if *v != b'#' {
                            Some((1, p2.manhattan_distance(&goal), p2))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }).collect())
            }
        });

        assert_eq!(res, Some((24, goal)));
    }

    #[test]
    fn bfs2_maze() {
        const BFS2_TABLE: &'static [(u8, Option<u32>)] = &[
            (b'x', Some(24)),
            (b'a', Some(0)),
            (b'b', Some(15)),
            (b'y', None),
        ];

        let maze = VecGrid::parse_lines(MAZE_2, b'\n').unwrap();
        let mut bfs: BFS<Point<usize>, u8> = BFS::new();

        for (target, expected) in BFS2_TABLE.iter() {
            let result = bfs.run(Point(1, 1), |p| {
                if let Some(c) = maze.get(p) {
                    if c.eq(target) {
                        return BFSResult::Success;
                    }

                    BFSResult::Continue(p.cardinals().iter().filter(|p| {
                        maze.get(*p) != Some(&b'#')
                    }).copied().collect())
                } else {
                    BFSResult::DeadEnd
                }
            });

            assert_eq!(result.map(|r| r.1), *expected);
        }
    }

    #[test]
    fn bfs2_maze_goals() {
        let maze = VecGrid::parse_lines(MAZE_2, b'\n').unwrap();
        let mut bfs: BFS<Point<usize>, u8> = BFS::new();

        bfs.run(Point(1, 1), |p| {
            if let Some(c) = maze.get(p) {
                let next_steps = p.cardinals().iter().filter(|p| {
                    maze.get(*p) != Some(&b'#')
                }).copied().collect();

                if *c >= b'a' && *c <= b'z' {
                    BFSResult::Found(*c, next_steps)
                } else {
                    BFSResult::Continue(next_steps)
                }
            } else {
                BFSResult::DeadEnd
            }
        });

        assert_eq!(bfs.found_goals(), &[
            (b'a', 0), (b'z', 10), (b'b', 15), (b'i', 17),
            (b'r', 20), (b't', 21), (b'x', 24),
        ]);
    }
}