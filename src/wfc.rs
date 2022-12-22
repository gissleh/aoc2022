use std::ops::{BitAndAssign, Not};
use num::PrimInt;
use crate::geo::Point;
use crate::grid2::{FixedGrid, GetterMutGrid, RowGrid};

trait State<K, M>: Sized {
    fn next(&self) -> Option<(K, M)>;
}

impl<M, G> State<Point<usize>, M> for G where G: GetterMutGrid<M> + FixedGrid, M: PrimInt + Copy {
    fn next(&self) -> Option<(Point<usize>, M)> {
        Point::range(self.width(), self.height())
            .map(|p| (p, self.get(&p).unwrap()))
            .filter(|(_, m)| m.count_ones() > 1)
            .min_by(|(_, a), (_, b)| a.count_ones().cmp(&b.count_ones()))
            .map(|(p, m)| (p, *m))
    }
}

trait Behavior<K, M, S>: Sized {
    fn collapse(&self, state: &mut S, key: &K, mask: M);

    fn and<B2>(self, b2: B2) -> And<Self, B2> where B2: Behavior<K, M, S> {
        And(self, b2)
    }
}

struct And<B1, B2> (B1, B2);

impl<B1, B2, K, M, S> Behavior<K, M, S> for And<B1, B2>
    where B1: Behavior<K, M, S>, B2: Behavior<K, M, S>, M: Copy {
    fn collapse(&self, state: &mut S, key: &K, mask: M) {
        self.0.collapse(state, key, mask);
        self.1.collapse(state, key, mask);
    }
}

#[derive(Copy, Clone)]
pub struct OnePerRow;

impl<M, S> Behavior<Point<usize>, M, S> for OnePerRow
    where M: BitAndAssign + Not<Output=M> + Copy, S: RowGrid<M> {
    fn collapse(&self, state: &mut S, key: &Point<usize>, mask: M) {
        let row = state.row_mut(*key.y()).unwrap();
        for current in row.iter_mut() {
            *current &= !mask;
        }
        row[key.0] = mask;
    }
}
