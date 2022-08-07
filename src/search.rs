use std::collections::VecDeque;
use std::hash::Hash;
use std::marker::PhantomData;
use std::ops::Add;
use rustc_hash::{FxHashMap, FxHashSet};

pub trait AStarState<B, G, C>: Eq + Hash + Clone where C: Ord + Copy + Clone + Add<Output=C> {
    fn heuristic(&self, board: &B, goal: &G) -> C;
    fn is_goal(&self, board: &B, goal: &G) -> bool;
    fn next(&self, board: &B, buffer: &mut Vec<(C, Self)>);
}

/// A Dijkstra/AStar implementation that uses a state trait to determine the next steps. If heuristic returns anything other than
/// 0, it's A*, otherwise it's plain Dijkstra. The state could be anything that satisfies AStarState and its supertraits. The Board
/// is an object that is pushed to the states.
pub fn astar<B, G, C, S>(board: &B, goal: &G, initial_state: S, initial_cost: C, cost_only_increases: bool, stop_after_first_goal: bool) -> Option<(C, S)>
    where
        C: Ord + Copy + Clone + Add<Output=C>,
        S: AStarState<B, G, C>
{
    let mut seen: FxHashMap<S, C> = FxHashMap::default();
    let mut unexplored: Vec<(C, S)> = Vec::with_capacity(32);
    let mut buffer: Vec<(C, S)> = Vec::with_capacity(8);
    let mut lowest: Option<(C, S)> = None;
    let mut next: (C, S) = (initial_cost, initial_state.clone());

    seen.insert(initial_state, initial_cost);

    loop {
        let (cost, state) = next;

        if state.is_goal(board, goal) {
            if stop_after_first_goal {
                lowest = Some((cost, state.clone()));
                break;
            } else {
                if let Some((lowest_cost, _)) = lowest {
                    if cost < lowest_cost {
                        lowest = Some((cost, state.clone()));
                    }
                } else {
                    lowest = Some((cost, state.clone()));
                }
            }
        } else {
            buffer.clear();
            state.next(board, &mut buffer);

            for (cost_increment, next_state) in buffer.iter() {
                let next_cost = cost + *cost_increment;

                if cost_only_increases {
                    if let Some((lowest_cost, _)) = &lowest {
                        if *lowest_cost <= next_cost {
                            continue;
                        }
                    }
                }

                if let Some(existing_cost) = seen.get(&next_state) {
                    if *existing_cost <= next_cost {
                        continue;
                    }
                }

                unexplored.push((next_cost, next_state.clone()));
                seen.insert(next_state.clone(), next_cost);
            }
        }

        if unexplored.is_empty() {
            break;
        }

        let mut winner_index = 0usize;
        let mut lowest_cost = unexplored[0].0 + unexplored[0].1.heuristic(board, goal);

        for (i, (next_cost, next_state)) in unexplored.iter().enumerate().skip(1) {
            let estimated_cost = *next_cost + next_state.heuristic(board, goal);
            if estimated_cost < lowest_cost {
                winner_index = i;
                lowest_cost = estimated_cost;
            }
        }

        next = unexplored.swap_remove(winner_index);
    }

    lowest
}

pub trait BFSState<B, G>: Eq + Hash + Clone {
    fn is_goal(&self, board: &B, goal: &G) -> bool;
    fn next(&self, board: &B, buffer: &mut Vec<Self>);
}

pub trait BFSExploreState<B, G>: Eq + Hash + Clone {
    fn has_goal(&self, board: &B) -> Option<G>;
}

/// BFS keeps track of states
pub struct BFS<B, G, S> where S: BFSState<B, G> {
    queue: VecDeque<(S, u32)>,
    buffer: Vec<S>,
    seen: FxHashSet<S>,
    goals: Vec<(G, u32)>,
    spooky: PhantomData<B>,
}

impl<B, G, S> BFS<B, G, S> where S: BFSState<B, G> {
    pub fn run(&mut self, board: &B, goal: &G, initial_state: S) -> Option<u32> {
        self.seen.clear();
        self.queue.clear();

        self.seen.insert(initial_state.clone());
        self.queue.push_back((initial_state, 0));

        while !self.queue.is_empty() {
            let (current_state, current_steps) = self.queue.pop_front().unwrap();

            if current_state.is_goal(board, goal) {
                return Some(current_steps);
            }

            current_state.next(board, &mut self.buffer);
            self.process_new_steps(current_steps);
        }

        None
    }

    fn process_new_steps(&mut self, current_steps: u32) {
        for next_state in self.buffer.iter() {
            if self.seen.contains(next_state) {
                continue;
            }

            self.seen.insert(next_state.clone());
            self.queue.push_back((next_state.clone(), current_steps + 1));
        }
        self.buffer.clear();
    }

    pub fn new() -> Self {
        Self {
            queue: VecDeque::with_capacity(64),
            buffer: Vec::with_capacity(8),
            seen: FxHashSet::default(),
            goals: Vec::new(),
            spooky: PhantomData::default(),
        }
    }
}

impl<B, G, S> BFS<B, G, S> where S: BFSState<B, G> + BFSExploreState<B, G> {
    /// Explore the entire board and note down everything that satisfies the has_goal method
    /// in the BFSExplorerState trait. It will stop when there are no more unseen paths to explore.
    /// The goals will be listed along with their shortest amount of steps from the initial state.
    pub fn explore(&mut self, board: &B, initial_state: S) {
        self.seen.clear();
        self.queue.clear();
        self.goals.clear();

        self.seen.insert(initial_state.clone());
        self.queue.push_back((initial_state, 0));

        while !self.queue.is_empty() {
            let (current_state, current_steps) = self.queue.pop_front().unwrap();

            if let Some(other_goal) = current_state.has_goal(board) {
                self.goals.push((other_goal, current_steps))
            }

            current_state.next(board, &mut self.buffer);
            self.process_new_steps(current_steps);
        }
    }

    /// Iterate over the goals from the last run.
    pub fn found_goals(&self) -> &[(G, u32)] {
        &self.goals
    }
}

#[cfg(test)]
mod tests {
    use crate::geo::Point;
    use crate::grid2::{FixedGrid, GetterGrid, VecGrid};
    use super::*;

    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    struct Knight(i32, i32);

    impl AStarState<(), Point<i32>, i32> for Knight {
        fn heuristic(&self, _board: &(), target: &Point<i32>) -> i32 {
            ((self.0 - target.0).abs() + (self.1 - target.1).abs()) as i32
        }

        fn is_goal(&self, _board: &(), target: &Point<i32>) -> bool {
            *self == Knight(target.0, target.1)
        }

        fn next(&self, _board: &(), buffer: &mut Vec<(i32, Self)>) {
            let Knight(x, y) = *self;

            println!("Checking next for {:?}", self);

            buffer.push((1, Knight(x + 1, y + 2)));
            buffer.push((1, Knight(x + 1, y - 2)));
            buffer.push((1, Knight(x + 2, y + 1)));
            buffer.push((1, Knight(x - 2, y + 1)));
            buffer.push((1, Knight(x - 1, y + 2)));
            buffer.push((1, Knight(x - 1, y - 2)));
            buffer.push((1, Knight(x + 2, y - 1)));
            buffer.push((1, Knight(x - 2, y - 1)));
        }
    }

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

    #[derive(Debug, Eq, PartialEq, Hash, Clone)]
    struct MazeWalker {
        pos: Point<usize>,
    }

    impl MazeWalker {
        fn try_direction<G>(&self, maze: &G, new_pos: Point<usize>, dst: &mut Vec<(usize, Self)>) where G: GetterGrid<u8> + FixedGrid {
            if let Some(ch) = maze.get(&new_pos) {
                if *ch == b'.' {
                    dst.push((1, MazeWalker { pos: new_pos }));
                }
            }
        }
    }

    impl<G> AStarState<G, Point<usize>, usize> for MazeWalker where G: GetterGrid<u8> + FixedGrid {
        fn heuristic(&self, _maze: &G, goal: &Point<usize>) -> usize {
            self.pos.distance(goal)
        }

        fn is_goal(&self, _maze: &G, goal: &Point<usize>) -> bool {
            self.pos.eq(goal)
        }

        fn next(&self, maze: &G, dst: &mut Vec<(usize, Self)>) {
            let Point(x, y) = self.pos;

            println!("Checking {},{} of {},{}", x, y, maze.width(), maze.height());

            self.try_direction(maze, Point(x + 1, y), dst);
            self.try_direction(maze, Point(x - 1, y), dst);
            self.try_direction(maze, Point(x, y + 1), dst);
            self.try_direction(maze, Point(x, y - 1), dst);
        }
    }

    #[test]
    fn chess_piece() {
        assert_eq!(astar(&(), &Point(4, 6), Knight(0, 0), 0, true, true), Some((4, Knight(4, 6))));
    }

    #[test]
    fn maze_walker() {
        let maze = VecGrid::parse_lines(MAZE, b'\n').unwrap();
        let walker = MazeWalker { pos: Point(1, 1) };

        let res = astar(&maze, &Point(maze.width() - 2, maze.height() - 2), walker, 0, true, true);
        assert_eq!(res.is_some(), true);
        let (cost, final_state) = res.unwrap();
        assert_eq!(cost, 24);
        assert_eq!(final_state.pos, Point(15, 7));
    }

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

    #[test]
    fn bfs_maze() {
        let maze = VecGrid::parse_lines(MAZE_2, b'\n').unwrap();
        let mut bfs = BFS::new();

        assert_eq!(bfs.run(&maze, &b'x', MazeBFS{pos: Point(1, 1)}), Some(24));
        assert_eq!(bfs.run(&maze, &b'a', MazeBFS{pos: Point(1, 1)}), Some(0));
        assert_eq!(bfs.run(&maze, &b'b', MazeBFS{pos: Point(1, 1)}), Some(15));
        assert_eq!(bfs.run(&maze, &b'y', MazeBFS{pos: Point(1, 1)}), None);

        bfs.explore(&maze, MazeBFS{pos: Point(1, 1)});
        assert_eq!(bfs.found_goals(), &[
            (b'a', 0), (b'z', 10), (b'b', 15), (b'i', 17),
            (b'r', 20), (b't', 21), (b'x', 24),
        ]);
    }

    #[derive(Hash, Eq, PartialEq, Clone)]
    struct MazeBFS {
        pos: Point<usize>,
    }

    impl MazeBFS {
        fn try_direction<G>(&self, maze: &G, new_pos: Point<usize>, buffer: &mut Vec<Self>) where G: GetterGrid<u8> {
            if let Some(ch) = maze.get(&new_pos) {
                if *ch != b'#' {
                    buffer.push(Self { pos: new_pos });
                }
            }
        }
    }

    impl<B> BFSState<B, u8> for MazeBFS where B: GetterGrid<u8> {
        fn is_goal(&self, maze: &B, goal: &u8) -> bool {
            if let Some(v) = maze.get(&self.pos) {
                v.eq(goal)
            } else {
                false
            }
        }

        fn next(&self, maze: &B, buffer: &mut Vec<Self>) {
            let Point(x, y) = self.pos;

            self.try_direction(maze, Point(x + 1, y), buffer);
            self.try_direction(maze, Point(x - 1, y), buffer);
            self.try_direction(maze, Point(x, y + 1), buffer);
            self.try_direction(maze, Point(x, y - 1), buffer);
        }
    }

    impl<B> BFSExploreState<B, u8> for MazeBFS where B: GetterGrid<u8> {
        fn has_goal(&self, board: &B) -> Option<u8> {
            if let Some(v) = board.get(&self.pos) {
                match *v {
                    b'#' | b'.' => None,
                    _ => Some(*v),
                }
            } else {
                None
            }
        }
    }
}