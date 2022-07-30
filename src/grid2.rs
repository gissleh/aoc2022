use crate::geo::Point;

pub struct ArrayGrid<T, const S: usize, const W: usize> {
    data: [T; S],
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
    pub fn new() -> ArrayGrid<T, S, W> {
        assert_eq!(S % W, 0);
        ArrayGrid {
            data: [Default::default(); S]
        }
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
        SliceIter { width: W, pos: 0, data: &self.data }
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

pub trait IterableSliceGrid<T> {
    /// Cells iterates over all grid cells with the positions
    fn cells(&self) -> SliceIter<'_, T>;
}

pub struct SliceIter<'a, T> {
    data: &'a [T],
    pos: usize,
    width: usize,
}

impl<'a, T> Iterator for SliceIter<'a, T> {
    type Item = (Point<usize>, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.get(self.pos) {
            Some(v) => {
                let x = self.pos % self.width;
                let y = self.pos / self.width;
                self.pos += 1;

                Some((Point(x, y), v))
            }
            None => None
        }
    }
}

#[test]
fn test_array_grid() {
    let mut ag: ArrayGrid<i32, 160, 16> = ArrayGrid::new_with(0);
    assert_eq!(ag.width(), 16);
    assert_eq!(ag.height(), 10);

    *ag.get_mut(&Point(14, 3)).unwrap() = 64;
    *ag.get_mut(&Point(11, 0)).unwrap() = 112;

    assert_eq!(ag.cells().count(), 160);
    assert_eq!(ag.row(0), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 112, 0, 0, 0, 0].as_slice()));
    assert_eq!(ag.row(1), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_slice()));
    assert_eq!(ag.row(2), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_slice()));
    assert_eq!(ag.row(3), Some([0i32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 64, 0].as_slice()));
    assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 64 && *x == 14 && *y == 3).is_some(), true);
    assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 112 && *x == 11 && *y == 0).is_some(), true);
    assert_eq!(ag.cells().find(|(Point(x, y), i)| **i == 175 && *x == 10 && *y == 1).is_some(), false);
}
