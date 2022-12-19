use std::collections::VecDeque;
use std::hash::Hash;
use std::marker::PhantomData;
use rustc_hash::FxHashSet;

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
