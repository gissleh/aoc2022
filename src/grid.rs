use crate::geo::Point;
use std::ops::{Index, IndexMut, Range};
use smallvec::{SmallVec, smallvec, Array};
use std::marker::PhantomData;

pub trait GridStorage<T>: ReadOnlyGridStorage<T> + IndexMut<usize, Output=T> {}

pub trait ReadOnlyGridStorage<T>: Index<usize, Output=T> + Clone + AsRef<[T]> {}

/// What is AoC without grids? To keep things generic, use the dummy traits `GridStorage<T>` and
/// `ReadOnlyGridStorage<T>` which contain the clunky traits needed for a full-featured grid.
///
/// ```
/// use common::grid::{ReadOnlyGridStorage, Grid};
/// use common::geo::Point;
///
/// pub fn part1<S: ReadOnlyGridStorage<char>>(input: &Grid<char, S>) -> u32 {
///     input.iter().fold(0u32, |c, (Point(x, y), v)| {
///         if x > 4 && y < 8 && *v == '#' {
///             c + 1
///         } else {
///             c
///         }
///     })
/// }
/// ```
pub struct Grid<T, S> {
    data: S,
    width: usize,
    height: usize,
    phantom: PhantomData<T>,
}

impl<T, S> Clone for Grid<T, S> where S: Clone, T: Clone {
    fn clone(&self) -> Self {
        Grid {
            data: self.data.clone(),
            width: self.width,
            height: self.height,
            phantom: PhantomData,
        }
    }
}

impl<T, S> Copy for Grid<T, S> where S: Copy, T: Copy {}

impl<T, S> Grid<T, S> where S: Index<usize, Output=T> {
    pub fn cell(&self, p: Point<usize>) -> Option<&T> {
        if p.0 < self.width && p.1 < self.height {
            Some(&self.data[p.0 * self.width + p.1])
        } else {
            None
        }
    }
}

impl<'a, T: 'a, S> Grid<T, S> where S: AsRef<[T]> {
    pub fn slice(&'a self) -> &'a [T] {
        self.data.as_ref()
    }

    pub fn iter(&'a self) -> impl Iterator<Item=(Point<usize>, &'a T)> {
        self.data.as_ref().iter().enumerate().map(|(i, v)| {
            (Point(i % self.width, i / self.width), v)
        })
    }

    pub fn print(&'a self, cb: fn(v: &T) -> (char, Option<String>)) {
        let mut buffer: Vec<String> = Vec::new();

        for (Point(_, y), v) in self.iter() {
            if y == 0 {
                for s in buffer.iter() {
                    print!(" {}", s)
                }
                buffer.clear();
                println!();
            }

            let (c, s) = cb(v);
            if let Some(s) = s {
                buffer.push(s);
            }

            print!("{}", c);
        }

        println!();
    }
}

impl<T, const SIZE: usize> Grid<T, [T; SIZE]> {
    pub fn iter_arr(&self) -> impl Iterator<Item=(Point<usize>, &T)> {
        self.data.iter().enumerate().map(|(i, v)| {
            (Point(i % self.width, i / self.width), v)
        })
    }
}

impl<T, S> Grid<T, S> where S: IndexMut<usize, Output=T> {
    pub fn cell_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width && y < self.height {
            Some(&mut self.data[y * self.width + x])
        } else {
            None
        }
    }
}

impl<T, S> Grid<T, S> where S: Index<Range<usize>, Output=[T]> {
    pub fn row(&self, y: usize) -> Option<&[T]> {
        if y < self.height {
            let s = self.width * y;
            let e = s + self.width;

            Some(&self.data[s..e])
        } else {
            None
        }
    }
}

impl<T, S> Grid<T, S> where S: IndexMut<Range<usize>, Output=[T]> {
    pub fn row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y < self.height {
            let s = self.width * y;
            let e = s + self.width;

            Some(&mut self.data[s..e])
        } else {
            None
        }
    }

    /// Get a horizontal slice of data.
    pub fn subrow_mut(&mut self, p: Point<usize>, w: usize) -> Option<&mut [T]> {
        let Point(x, y) = p;

        if y < self.height && w < self.width && x < self.width - w {
            let s = self.width * y + x;
            let e = s + w;

            Some(&mut self.data[s..e])
        } else {
            None
        }
    }

    pub fn map_row<F>(&mut self, y: usize, f: F) -> bool where F: Fn(&mut T, Point<usize>) {
        if let Some(row) = self.row_mut(y) {
            for (i, v) in row.iter_mut().enumerate() {
                f(v, Point(i, y));
            }

            true
        } else {
            false
        }
    }

    pub fn map_subrow<F>(&mut self, p: Point<usize>, w: usize, f: F) -> bool where F: Fn(&mut T, Point<usize>) {
        if let Some(row) = self.subrow_mut(p, w) {
            for (i, v) in row.iter_mut().enumerate() {
                f(v, Point(p.x() + i, *p.y()));
            }

            true
        } else {
            false
        }
    }

    pub fn map_rect<F>(&mut self, p1: Point<usize>, p2: Point<usize>, f: F) -> bool where F: Fn(&mut T, Point<usize>) {
        let Point(x1, y1) = p1;
        let Point(x2, y2) = p2;
        if x2 <= x1 || y2 <= y1 {
            return false;
        }

        let w = x2 - x1;
        for y in y1..y2 {
            let s = self.width * y + x1;
            let e = s + w;

            for (i, v) in self.data[s..e].iter_mut().enumerate() {
                f(v, Point(x1 + i, y));
            }
        }

        false
    }
}

impl<T, S> Grid<T, S> where S: IndexMut<Range<usize>, Output=[T]>, T: Copy {
    pub fn fill_row(&mut self, y: usize, v: T) -> bool {
        if let Some(row) = self.row_mut(y) {
            for v2 in row.iter_mut() {
                *v2 = v;
            }

            true
        } else {
            false
        }
    }

    pub fn fill_subrow(&mut self, p: Point<usize>, w: usize, v: T) -> bool {
        if let Some(row) = self.subrow_mut(p, w) {
            for v2 in row.iter_mut() {
                *v2 = v;
            }

            true
        } else {
            false
        }
    }

    pub fn fill_rect(&mut self, p1: Point<usize>, p2: Point<usize>, v: T) -> bool {
        let Point(x1, y1) = p1;
        let Point(x2, y2) = p2;
        if x2 <= x1 || y2 <= y1 {
            return false;
        }

        let w = x2 - x1;
        for y in y1..y2 {
            let s = self.width * y + x1;
            let e = s + w;

            for v2 in self.data[s..e].iter_mut() {
                *v2 = v;
            }
        }

        false
    }
}

impl<T, S> Grid<T, S> where S: Index<usize, Output=T> {
    pub fn new(width: usize, height: usize, data: S) -> Self {
        let _ = data[(height * width) - 1];

        Grid {
            width,
            height,
            data,
            phantom: PhantomData,
        }
    }
}

impl<T, const S: usize> Grid<T, [T; S]> where T: Copy {
    pub fn new_arr(width: usize, initial_value: T) -> Self {
        assert_eq!(S % width, 0);

        Grid {
            data: [initial_value; S],
            width,
            height: S / width,
            phantom: PhantomData,
        }
    }
}

impl<T> Grid<T, Vec<T>> {
    pub fn from_vec(width: usize, height: usize, data: Vec<T>) -> Self {
        assert_eq!(data.len(), width * height);

        Grid {
            data,
            width,
            height,
            phantom: PhantomData,
        }
    }
}

impl<T> Grid<T, Vec<T>> where T: Copy {
    pub fn new_vec(width: usize, height: usize, initial_value: T) -> Self {
        Grid {
            data: vec![initial_value; width * height],
            width,
            height,
            phantom: PhantomData,
        }
    }
}

impl<T> Grid<T, Vec<T>> where T: Default {
    pub fn new_vec_default(width: usize, height: usize) -> Self {
        let mut vec = Vec::with_capacity(width * height);
        vec.resize_with(width * height, Default::default);

        Grid {
            data: vec,
            width,
            height,
            phantom: PhantomData,
        }
    }
}

impl<T, A: Array<Item=T>> Grid<T, SmallVec<A>> where T: Copy {
    pub fn new_smallvec(width: usize, height: usize, initial_value: T) -> Self {
        Grid {
            data: smallvec![initial_value; width * height],
            width,
            height,
            phantom: PhantomData,
        }
    }
}

impl<T, A: Array<Item=T>> Grid<T, SmallVec<A>> where T: Default {
    pub fn new_smallvec_default(width: usize, height: usize) -> Self {
        let mut vec = SmallVec::with_capacity(width * height);
        vec.resize_with(width * height, Default::default);

        Grid {
            data: vec,
            width,
            height,
            phantom: PhantomData,
        }
    }
}

pub type ArrayGrid<T, const S: usize> = Grid<T, [T; S]>;
pub type VecGrid<T> = Grid<T, Vec<T>>;
