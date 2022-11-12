use std::fmt::Display;
use std::ops::Add;
use num::traits::WrappingAdd;
use crate::geo::Point;

pub const NEIGHBORS: &'static [Point<usize>; 8] = &[
    Point(usize::MAX, usize::MAX),
    Point(0, usize::MAX),
    Point(1, usize::MAX),
    Point(usize::MAX, 0),
    Point(1, 0),
    Point(usize::MAX, 1),
    Point(0, 1),
    Point(1, 1),
];

pub struct ArrayGrid<T, const S: usize, const W: usize> {
    data: [T; S],
}

impl<T, const S: usize, const W: usize> Clone for ArrayGrid<T, S, W> where T: Clone {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl<T, const S: usize, const W: usize> CountableGrid<T> for ArrayGrid<T, S, W> where T: Eq {
    fn count_occurrences_of(&self, pred: &T) -> usize {
        self.data.iter().filter(|v| pred.eq(*v)).count()
    }

    fn count_occurrences_where<F>(&self, pred: F) -> usize where F: Fn(&T) -> bool {
        self.data.iter().filter(|v| pred(*v)).count()
    }
}

impl<T, const S: usize, const W: usize> NeighborCountGrid<T> for ArrayGrid<T, S, W>  where T: Eq + WrappingAdd + Add<Output=T> {
    fn count_neighbors(&self, pos: &Point<usize>, pred: &T) -> usize {
        let mut count = 0;
        for n in NEIGHBORS.iter() {
            let curr = pos.wrapping_add(n);
            if self.get(&curr).contains(&pred) {
                count += 1;
            }
        }

        count
    }

    fn count_neighbors_where<F>(&self, pos: &Point<usize>, pred: F) -> usize where F: Fn(&T) -> bool {
        let mut count = 0usize;
        for n in NEIGHBORS.iter() {
            let curr = pos.wrapping_add(n);
            if let Some(v) = self.get(&curr) {
                if pred(v) {
                    count += 1;
                }
            }
        }

        count
    }
}

impl<T, const S: usize, const W: usize> ClearableGrid for ArrayGrid<T, S, W> where T: Copy + Default {
    fn clear(&mut self) {
        self.data.fill(T::default());
    }
}

impl<T, const S: usize, const W: usize> FillableGrid<T> for ArrayGrid<T, S, W> where T: Copy {
    fn fill(&mut self, v: T) {
        self.data.fill(v);
    }
}

impl<T, const S: usize, const W: usize> ArrayGrid<T, S, W> where T: Copy {
    pub fn new_with(initial_value: T) -> ArrayGrid<T, S, W> {
        assert_eq!(S % W, 0);
        ArrayGrid {
            data: [initial_value; S]
        }
    }

    pub fn cells_nobox(&self) -> impl Iterator<Item=(Point<usize>, &T)> {
        self.data.iter().enumerate().map(|(i, v)| {
            (Point(i % W, i / W), v)
        })
    }
}

impl<T, const S: usize, const W: usize> ArrayGrid<T, S, W> where T: Default + Copy {
    pub fn new() -> Self {
        assert_eq!(S % W, 0);
        Self {
            data: [Default::default(); S]
        }
    }

    pub fn from_array(data: [T; S]) -> Self {
        assert_eq!(S % W, 0);
        Self { data }
    }
}

impl<T, const S: usize, const W: usize> FixedGrid for ArrayGrid<T, S, W> {
    fn width(&self) -> usize {
        W
    }
    fn height(&self) -> usize {
        S / W
    }
}

impl<T, const S: usize, const W: usize> GetterGrid<T> for ArrayGrid<T, S, W> {
    fn get(&self, pos: &Point<usize>) -> Option<&T> {
        if pos.0 >= W {
            return None;
        }

        self.data.get(pos.1 * W + pos.0)
    }

    fn get_mut(&mut self, pos: &Point<usize>) -> Option<&mut T> {
        if pos.0 >= W {
            return None;
        }

        self.data.get_mut(pos.1 * W + pos.0)
    }
}

impl<T, const S: usize, const W: usize> IterableSliceGrid<T> for ArrayGrid<T, S, W> {
    fn cells(&self) -> SliceIter<'_, T> {
        SliceIter::new(&self.data, W)
    }
}


impl<T, const S: usize, const W: usize> RowGrid<T> for ArrayGrid<T, S, W> {
    fn row(&self, y: usize) -> Option<&[T]> {
        if y * W >= S {
            return None;
        }

        Some(&self.data[y * W..(y + 1) * W])
    }

    fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y * W >= S {
            return None;
        }

        Some(&mut self.data[y * W..(y + 1) * W])
    }
}

pub struct VecGrid<T> {
    data: Vec<T>,
    width: usize,
}

impl<T> FillableGrid<T> for VecGrid<T> where T: Copy {
    fn fill(&mut self, v: T) {
        self.data.fill(v);
    }
}

impl<T> ClearableGrid for VecGrid<T> where T: Default + Copy {
    fn clear(&mut self) {
        self.data.fill(T::default());
    }
}

impl<T> VecGrid<T> {
    pub fn new_from(width: usize, data: Vec<T>) -> Self {
        assert!(data.len() >= width);
        assert_eq!(data.len() % width, 0);

        VecGrid { data, width }
    }
}

impl<T> VecGrid<T> where T: Eq + Copy {
    pub fn parse_lines(raw: &[T], newline: T) -> Option<Self> {
        let width = raw.iter().take_while(|v| **v != newline).count();
        let data: Vec<T> = raw.iter().filter(|v| **v != newline).copied().collect();

        if data.len() % width == 0 {
            Some(VecGrid { data, width })
        } else {
            None
        }
    }
}


impl<T> VecGrid<T> where T: Copy {
    pub fn new_with(width: usize, height: usize, v: T) -> Self {
        VecGrid { data: vec![v; width * height], width }
    }
}

impl<T> VecGrid<T> where T: Copy + Default {
    pub fn new(width: usize, height: usize) -> Self {
        VecGrid { data: vec![Default::default(); width * height], width }
    }
}

impl<T> FixedGrid for VecGrid<T> {
    fn width(&self) -> usize {
        self.width
    }
    fn height(&self) -> usize {
        self.data.len() / self.width
    }
}

impl<T> GetterGrid<T> for VecGrid<T> {
    fn get(&self, pos: &Point<usize>) -> Option<&T> {
        if pos.0 >= self.width {
            return None;
        }

        self.data.get(pos.1 * self.width + pos.0)
    }

    fn get_mut(&mut self, pos: &Point<usize>) -> Option<&mut T> {
        if pos.0 >= self.width {
            return None;
        }

        self.data.get_mut(pos.1 * self.width + pos.0)
    }
}


impl<T> RowGrid<T> for VecGrid<T> {
    fn row(&self, y: usize) -> Option<&[T]> {
        if y * self.width > self.data.len() {
            return None;
        }

        Some(&self.data[y * self.width..(y + 1) * self.width])
    }

    fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y * self.width > self.data.len() {
            return None;
        }

        Some(&mut self.data[y * self.width..(y + 1) * self.width])
    }
}

impl<T> IterableSliceGrid<T> for VecGrid<T> {
    fn cells(&self) -> SliceIter<'_, T> {
        return SliceIter::new(&self.data, self.width);
    }
}

pub trait FixedGrid {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
}

pub trait GetterGrid<T> {
    fn get(&self, pos: &Point<usize>) -> Option<&T>;
    fn get_mut(&mut self, pos: &Point<usize>) -> Option<&mut T>;
}

pub trait RowGrid<T> {
    fn row(&self, y: usize) -> Option<&[T]>;
    fn row_mut(&mut self, y: usize) -> Option<&mut [T]>;
}

pub trait NeighborCountGrid<T> {
    fn count_neighbors(&self, pos: &Point<usize>, pred: &T) -> usize;
    fn count_neighbors_where<F>(&self, pos: &Point<usize>, pred: F) -> usize where F: Fn(&T) -> bool;
}

pub trait CountableGrid<T> {
    fn count_occurrences_of(&self, pred: &T) -> usize;
    fn count_occurrences_where<F>(&self, pred: F) -> usize where F: Fn(&T) -> bool;
}

pub trait IterableSliceGrid<T> {
    /// Cells iterates over all grid cells with the positions
    fn cells(&self) -> SliceIter<'_, T>;

    /// Print a grid based on the callback. The second argument is
    /// for side-data that will be appended at the end of a line, like
    /// the gnomes and elves in Beverage Bandit.
    fn print<F, D1, D2>(&self, f: F)
        where F: Fn(&T) -> (D1, Option<D2>),
              D1: Display,
              D2: Display {
        let mut vec: Vec<D2> = Vec::new();

        for (Point(x, y), v) in self.cells() {
            if x == 0 && y != 0 {
                for item in vec.drain(0..) {
                    print!(" {}", item);
                }

                println!();
            }

            let (d1, d2) = f(v);

            print!("{}", d1);
            if let Some(d2) = d2 {
                vec.push(d2);
            }
        }

        for item in vec.drain(0..) {
            print!("{} ", item);
        }
        println!();
    }
}

pub trait ClearableGrid {
    fn clear(&mut self);
}

pub trait FillableGrid<T> where T: Copy {
    fn fill(&mut self, v: T);
}

pub struct SliceIter<'a, T> {
    data: &'a [T],
    pos: usize,
    width: usize,
    x: usize,
    y: usize,
}

impl<'a, T> SliceIter<'a, T> {
    fn new(data: &'a [T], width: usize) -> Self {
        SliceIter {
            data,
            width,
            pos: 0,
            x: 0,
            y: 0,
        }
    }
}

impl<'a, T> Iterator for SliceIter<'a, T> {
    type Item = (Point<usize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.get(self.pos) {
            Some(v) => {
                let x = self.x;
                let y = self.y;

                self.pos += 1;
                self.x += 1;
                if self.x == self.width {
                    self.x = 0;
                    self.y += 1;
                }

                Some((Point(x, y), v))
            }
            None => None
        }
    }

    fn count(self) -> usize where Self: Sized {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_array_grid() {
        let mut ag: ArrayGrid<i32, 160, 16> = ArrayGrid::new_with(0);
        assert_eq!(ag.width(), 16);
        assert_eq!(ag.height(), 10);

        *ag.get_mut(&Point(14, 3)).unwrap() = 64;
        *ag.get_mut(&Point(11, 0)).unwrap() = 112;

        assert_eq!(ag.cells().count(), 160);
        assert_eq!(ag.cells().filter(|_| true).count(), 160);
        assert_eq!(ag.row(0), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 0, 0, 0, 0].as_slice()));
        assert_eq!(ag.row(1), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_slice()));
        assert_eq!(ag.row(2), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_slice()));
        assert_eq!(ag.row(3), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0].as_slice()));
        assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 64 && *x == 14 && *y == 3).is_some(), true);
        assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 112 && *x == 11 && *y == 0).is_some(), true);
        assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 175 && *x == 10 && *y == 1).is_some(), false);
    }
}
