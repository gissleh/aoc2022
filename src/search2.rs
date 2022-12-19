use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use rustc_hash::{FxHashMap, FxHashSet};

pub trait Search<S: Sized>: Sized {
    fn reset(&mut self);
    fn next_step(&mut self) -> Option<S>;
    fn add_step(&mut self, step: S);

    /// Add a filter to the search that is applied on states before they're run. No state
    /// will be evaluated twice with the bounds check.
    fn bounded<F: Fn(&S) -> bool>(self, f: F) -> Bounded<Self, F> {
        Bounded(self, f)
    }

    fn run<R, F: FnMut(&mut Self, &S) -> Option<R>>(self, f: F) -> Run<Self, F, S, R> {
        Run(self, f, PhantomData::default())
    }
}

pub struct Run<SEARCH, F, S, R> (SEARCH, F, PhantomData<(R, S)>);

impl<SEARCH, F, S, R> Iterator for Run<SEARCH, F, S, R>
    where SEARCH: Search<S>,
          F: FnMut(&mut SEARCH, &S) -> Option<R> {
    type Item = R;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(s) = self.0.next_step() {
            if let Some(r) = self.1(&mut self.0, &s) {
                return Some(r);
            }
        }

        None
    }
}

pub struct Bounded<SEARCH, F> (SEARCH, F);

impl<S, SEARCH, F> Search<S> for Bounded<SEARCH, F>
    where SEARCH: Search<S>,
          F: Fn(&S) -> bool {
    #[inline]
    fn reset(&mut self) { self.0.reset() }
    fn next_step(&mut self) -> Option<S> {
        if let Some(next_step) = self.0.next_step() {
            if self.1(&next_step) {
                Some(next_step)
            } else {
                self.next_step()
            }
        } else {
            None
        }
    }
    #[inline]
    fn add_step(&mut self, step: S) { self.0.add_step(step) }
}

struct DFS<S> {
    stack: Vec<S>,
    seen: FxHashSet<S>,
}

impl<S> Search<S> for DFS<S> where S: Hash + Eq + Clone {
    fn reset(&mut self) {
        self.stack.clear();
        self.seen.clear();
    }

    fn next_step(&mut self) -> Option<S> {
        self.stack.pop()
    }

    fn add_step(&mut self, step: S) {
        if self.seen.insert(step.clone()) {
            self.stack.push(step);
        }
    }
}

pub fn dfs<S>(initial_step: S) -> impl Search<S> where S: Default + Hash + Eq + Clone {
    let mut dfs = DFS { stack: Vec::with_capacity(64), seen: FxHashSet::default() };
    dfs.add_step(initial_step);
    dfs
}

struct BFS<S> {
    queue: VecDeque<S>,
    seen: FxHashSet<S>,
}

impl<S> Search<S> for BFS<S> where S: Hash + Eq + Clone {
    fn reset(&mut self) {
        self.queue.clear();
        self.seen.clear();
    }

    fn next_step(&mut self) -> Option<S> {
        self.queue.pop_front()
    }

    fn add_step(&mut self, step: S) {
        if self.seen.insert(step.clone()) {
            self.queue.push_back(step);
        }
    }
}

pub fn bfs<S>(initial_step: S) -> impl Search<S> where S: Default + Hash + Eq + Clone {
    let mut bfs = BFS { queue: VecDeque::with_capacity(64), seen: FxHashSet::default() };
    bfs.add_step(initial_step);
    bfs
}

struct Dijkstra<C, K, S> where C: Ord + Eq, K: Hash + Eq, S: DijkstraState<C, K> {
    initial_step: S,
    seen: FxHashMap<K, C>,
    open: BinaryHeap<DijkstraStep<C, K, S>>,
}

impl<C, H, S> Search<S> for Dijkstra<C, H, S> where C: Ord + Eq, H: Hash + Eq + PartialEq, S: DijkstraState<C, H> {
    fn reset(&mut self) {
        self.open.clear();
        self.seen.clear();

        self.open.push(DijkstraStep(
            self.initial_step.cost(),
            self.initial_step.clone(),
            PhantomData::default())
        );
        self.seen.insert(self.initial_step.key(), self.initial_step.cost());
    }

    fn next_step(&mut self) -> Option<S> {
        self.open.pop().map(|DijkstraStep(_, s, _)| s)
    }

    fn add_step(&mut self, step: S) {
        let seen_key = step.key();
        let step_cost = step.cost();
        if let Some(seen_cost) = self.seen.get(&seen_key) {
            if step_cost >= *seen_cost {
                return;
            }
        }
        self.seen.insert(seen_key, step_cost);

        self.open.push(DijkstraStep(
            step.cost(), step,
            PhantomData::default())
        );
    }
}

struct DijkstraStep<C, K, S> (C, S, PhantomData<K>);

impl<C, K, S> Eq for DijkstraStep<C, K, S> where C: Eq + Ord, K: Eq + Hash, S: DijkstraState<C, K> {}

impl<C, K, S> PartialEq<Self> for DijkstraStep<C, K, S> where C: Eq + Ord, K: Eq + Hash, S: DijkstraState<C, K> {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<C, K, S> PartialOrd<Self> for DijkstraStep<C, K, S> where C: Eq + Ord, K: Eq + Hash, S: DijkstraState<C, K> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl<C, K, S> Ord for DijkstraStep<C, K, S> where S: DijkstraState<C, K>, C: Ord + Eq, K: Hash + Eq {
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

pub trait DijkstraState<C: Ord + Eq, K: Hash + Eq>: Clone {
    fn cost(&self) -> C;
    fn key(&self) -> K;
}

/// WithCost is a wrapper type for a state that can be used for bfs, dfs and dijkstra that will
/// allow you to associate cost without it making dfs/bfs run forever.
#[derive(Clone, Copy, Default)]
pub struct WithCost<S, C> (S, C);

impl<S, C> PartialEq<Self> for WithCost<S, C> where S: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<S, C> Eq for WithCost<S, C> where S: PartialEq {}

impl<S, C> Hash for WithCost<S, C> where S: Hash + Eq {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<S, C> DijkstraState<C, S> for WithCost<S, C> where S: Hash + Eq + Clone, C: Ord + Eq + Copy + Clone {
    fn cost(&self) -> C {
        self.1
    }

    fn key(&self) -> S {
        self.0.clone()
    }
}

/// Run dijkstra search. The state needs to implement DijkstraState<T> and is not interchangeable
/// with bfs and dfs without changing up how costs and such are handled.
pub fn dijkstra<C, K, S>(initial_step: S) -> impl Search<S> where C: Ord + Eq, K: Hash + Eq, S: DijkstraState<C, K> {
    let mut dijkstra = Dijkstra {
        initial_step,
        seen: FxHashMap::default(),
        open: BinaryHeap::with_capacity(128),
    };

    dijkstra.reset();

    dijkstra
}


#[cfg(test)]
pub mod tests {
    use crate::geo::Point;
    use crate::grid2::{GetterGrid, VecGrid};
    use super::*;

    const MAZE_01: &[u8] = include_bytes!("./test_fixtures/search2_maze01.txt");

    fn parse_grid(grid: &[u8]) -> VecGrid<u8> {
        VecGrid::new_from(
            grid.iter().take_while(|b| **b != b'\n').count(),
            grid.iter().filter(|b| **b != b'\n').copied().collect(),
        )
    }

    #[test]
    fn dijkstra_maze() {
        fn run_search<S: Search<WithCost<Point<usize>, u32>>>(s: S) -> Vec<(char, u32)> {
            let maze_01 = parse_grid(MAZE_01);

            s.run(|search, WithCost(pos, cost)| {
                println!("{:?} {} {}", pos, *maze_01.get(&pos).unwrap() as char, cost);
                match *maze_01.get(&pos).unwrap() {
                    b'#' => None,
                    b'.' => {
                        for p in pos.cardinals_offset(1) {
                            search.add_step(WithCost(p, cost + 1));
                        }

                        None
                    }
                    ch => {
                        for p in pos.cardinals_offset(1) {
                            search.add_step(WithCost(p, cost + 1));
                        }

                        Some((ch as char, *cost))
                    }
                }
            }).collect::<Vec<_>>()
        }

        let initial_step = WithCost(Point(1, 1), 0u32);

        let results_dijkstra = run_search(dijkstra(initial_step));
        let results_bfs = run_search(bfs(initial_step));
        let results_dfs = run_search(dfs(initial_step));

        assert_eq!(results_dijkstra.as_slice(), &[('a', 51), ('b', 65), ('c', 71), ('z', 73)]);
        assert_eq!(results_bfs.as_slice(), &[('a', 51), ('b', 65), ('c', 71), ('z', 73)]);
        assert_eq!(results_dfs.len(), 4);
    }
}