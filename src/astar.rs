use std::hash::Hash;
use std::ops::Add;
use rustc_hash::FxHashMap;

pub trait AStarState<B, C>: Eq + Hash + Clone where C: Ord + Copy + Clone + Add<Output=C> {
    fn heuristic(&self, board: &B) -> C;
    fn is_goal(&self, board: &B) -> bool;
    fn next(&self, board: &B, buffer: &mut Vec<(C, Self)>);
}

fn astar<B, C, S>(board: &B, initial_state: S, initial_cost: C, exhaustive: bool) -> Option<(C, S)> where C: Ord + Copy + Clone + Add<Output=C>, S: AStarState<B, C> {
    let mut seen: FxHashMap<S, C> = FxHashMap::default();
    let mut unexplored: Vec<(C, S)> = Vec::with_capacity(32);
    let mut buffer: Vec<(C, S)> = Vec::with_capacity(8);
    let mut lowest: Option<(C, S)> = None;
    let mut next: (C, S) = (initial_cost, initial_state.clone());

    seen.insert(initial_state, initial_cost);

    loop {
        let (cost, state) = next;

        if state.is_goal(board) {
            lowest = Some((cost, state.clone()));
            if !exhaustive {
                break;
            }
        } else {
            buffer.clear();
            state.next(board, &mut buffer);

            for (cost_increment, next_state) in buffer.iter() {
                let next_cost = cost + *cost_increment;

                if let Some(existing_cost) = seen.get(&next_state) {
                    if *existing_cost <= next_cost {
                        continue;
                    }
                }
                if let Some((lowest_cost, _)) = &lowest {
                    if *lowest_cost <= next_cost {
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
        let mut lowest_cost = unexplored[0].0 + unexplored[0].1.heuristic(board);

        for (i, (next_cost, next_state)) in unexplored.iter().enumerate().skip(1) {
            let estimated_cost = *next_cost + next_state.heuristic(board);
            if estimated_cost < lowest_cost {
                winner_index = i;
                lowest_cost = estimated_cost;
            }
        }

        next = unexplored.swap_remove(winner_index);
    }

    lowest
}

#[cfg(test)]
mod tests {
    use std::hash::Hasher;
    use std::marker::PhantomData;
    use crate::geo::Point;
    use crate::grid2::{FixedGrid, GetterGrid, VecGrid};
    use super::*;

    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    struct Knight(i32, i32);

    impl AStarState<Point<i32>, i32> for Knight {
        fn heuristic(&self, target: &Point<i32>) -> i32 {
            ((self.0 - target.0).abs() + (self.1 - target.1).abs()) as i32
        }

        fn is_goal(&self, target: &Point<i32>) -> bool {
            *self == Knight(target.0, target.1)
        }

        fn next(&self, _: &Point<i32>, buffer: &mut Vec<(i32, Self)>) {
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

    impl<G> AStarState<G, usize> for MazeWalker where G: GetterGrid<u8> + FixedGrid {
        fn heuristic(&self, maze: &G) -> usize {
            let Point(x, y) = self.pos;

            maze.width() - 2 - x + maze.height() - 2 - y
        }

        fn is_goal(&self, maze: &G) -> bool {
            self.pos == Point(maze.width() - 2, maze.height() - 2)
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
        assert_eq!(astar(&Point(4, 6), Knight(0, 0), 0, false), Some((4, Knight(4, 6))));
    }

    #[test]
    fn maze_walker() {
        let maze = VecGrid::parse_lines(MAZE, b'\n').unwrap();
        let walker = MazeWalker { pos: Point(1, 1) };

        let res = astar(&maze, walker, 0, false);
        assert_eq!(res.is_some(), true);
        let (cost, final_state) = res.unwrap();
        assert_eq!(cost, 25);
        assert_eq!(final_state.pos, Point(15, 7));
    }
}