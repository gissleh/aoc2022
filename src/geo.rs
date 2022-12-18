use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::Step;
use arrayvec::ArrayVec;
use std::ops::{Add, Mul, Neg, Sub, Shr, Div, AddAssign, Rem, SubAssign};
use num::integer::{sqrt, Roots};
use num::{pow, One, Zero};
use num::traits::{WrappingAdd};
use crate::parse3::Parser;

#[derive(Hash, Eq, PartialEq)]
pub struct Point<T> (pub T, pub T);

impl<T> Sub for Point<T> where T: Sub<Output=T> {
    type Output = Point<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Point(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl<T> Point<T> {
    #[inline]
    pub fn x(&self) -> &T { &self.0 }
    #[inline]
    pub fn y(&self) -> &T { &self.1 }
}

impl<T> Point<T> {
    #[inline]
    pub fn with_z(self, z: T) -> Vertex<T> {
        Vertex(self.0, self.1, z)
    }
}

impl<T> Point<T> where T: Eq + Copy + Sub<Output=T> + AddAssign + SubAssign + Zero + One + Mul<Output=T> {
    #[inline]
    pub fn manhattan_diamond(&self, distance: T) -> impl Iterator<Item=Point<T>> {
        ManhattanDiamond {
            distance,

            round: 0,
            remaining: distance - T::one(),
            current: Point(self.0, self.1 - distance),
            zero: T::zero(),
        }
    }
}


impl<T> Point<T> where T: Ord {
    #[inline]
    pub fn side_of(&self, center: &Point<T>) -> usize {
        (if self.0 < center.0 { 0 } else { 1 }
            | if self.1 < center.1 { 0 } else { 2 })
    }
}

impl<T> Point<T> where T: Ord + PartialOrd + Zero + Add + Step + Copy {
    #[inline]
    pub fn range(w: T, h: T) -> impl Iterator<Item=Self> {
        (T::zero()..w).flat_map(move |x| (T::zero()..h).map(move |y| Point(x, y)))
    }
}

impl<T> Point<T> where T: Roots + Sub<Output=T> + Copy {
    pub fn distance(&self, rhs: &Point<T>) -> T {
        sqrt(
            pow(rhs.0 - self.0, 2)
                + pow(rhs.1 - self.1, 2)
        )
    }
}


impl<T> Point<T> where T: Sub<Output=T> + Add<Output=T> + Copy {
    /// Move the Point to a center based on side_of, for quadtree usage.
    pub fn move_center(&self, side: usize, distance: T) -> Point<T> {
        match side {
            0 => Point(self.0 - distance, self.1 - distance),
            1 => Point(self.0 + distance, self.1 - distance),
            2 => Point(self.0 - distance, self.1 + distance),
            3 => Point(self.0 + distance, self.1 + distance),
            _ => panic!("invalid side {}", side),
        }
    }
}

impl<T> Point<T> where T: Sub<Output=T> + Add<Output=T> + Copy + Ord {
    pub fn manhattan_distance(&self, rhs: &Point<T>) -> T {
        let a = if self.0 > rhs.0 { self.0 - rhs.0 } else { rhs.0 - self.0 };
        let b = if self.1 > rhs.1 { self.1 - rhs.1 } else { rhs.1 - self.1 };

        a + b
    }
}

impl<T> Point<T> where T: Copy + Add<Output=T> + Sub<Output=T> {
    #[inline]
    pub fn surrounding_rect_inclusive(&self, dist: T) -> Rect<T> {
        Rect(
            Point(self.0 - dist, self.1 - dist),
            Point(self.0 + dist, self.1 + dist),
        )
    }

    #[inline]
    pub fn cardinals_offset(&self, off: T) -> [Point<T>; 4] {
        [
            Point(self.0, self.1 - off),
            Point(self.0 - off, self.1),
            Point(self.0 + off, self.1),
            Point(self.0, self.1 + off),
        ]
    }
}

impl<T> Point<T> where T: One + Copy + Add<Output=T> + Sub<Output=T> {
    #[inline]
    pub fn surrounding_rect(&self, dist: T) -> Rect<T> {
        Rect(
            Point(self.0 - dist, self.1 - dist),
            Point(self.0 + dist + T::one(), self.1 + dist + T::one()),
        )
    }

    #[inline]
    pub fn cardinals(&self) -> [Point<T>; 4] {
        self.cardinals_offset(T::one())
    }

    #[inline]
    pub fn neighbors(&self) -> [Point<T>; 8] {
        [
            Point(self.0 - T::one(), self.1 - T::one()),
            Point(self.0, self.1 - T::one()),
            Point(self.0 + T::one(), self.1 - T::one()),
            Point(self.0 - T::one(), self.1),
            Point(self.0 + T::one(), self.1),
            Point(self.0 - T::one(), self.1 + T::one()),
            Point(self.0, self.1 + T::one()),
            Point(self.0 + T::one(), self.1 + T::one()),
        ]
    }

    #[inline]
    /// This returns an array of 4 neighboring coordinates.
    pub fn diagonals(&self) -> [Point<T>; 4] {
        [
            Point(self.0 - T::one(), self.1 - T::one()),
            Point(self.0 + T::one(), self.1 - T::one()),
            Point(self.0 - T::one(), self.1 + T::one()),
            Point(self.0 + T::one(), self.1 + T::one()),
        ]
    }
}

impl<T> Display for Point<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{}, {}>", self.0, self.1)
    }
}

impl<T> PartialOrd<Self> for Point<T> where T: Ord {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Point<T> where T: Ord {
    fn cmp(&self, other: &Self) -> Ordering {
        self.1.cmp(&other.1)
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl<T> AddAssign for Point<T> where T: AddAssign {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl<T> WrappingAdd for Point<T> where T: WrappingAdd + Add<Output=T> {
    fn wrapping_add(&self, v: &Self) -> Self {
        Point(self.0.wrapping_add(&v.0), self.1.wrapping_add(&v.1))
    }
}

impl<T> Copy for Point<T> where T: Copy {}

impl<T> Clone for Point<T> where T: Clone {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone())
    }
}

impl<T> Neg for Point<T> where T: Neg<Output=T> {
    type Output = Point<T>;

    fn neg(self) -> Self::Output {
        Point(self.0.neg(), self.1.neg())
    }
}

impl<T> Mul for Point<T> where T: Mul<Output=T> {
    type Output = Point<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Point(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl<T> Div for Point<T> where T: Div<Output=T> {
    type Output = Point<T>;

    fn div(self, rhs: Self) -> Self::Output {
        Point(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl<T> Add for Point<T> where T: Add<Output=T> {
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T> Rem for Point<T> where T: Rem<Output=T> {
    type Output = Point<T>;

    fn rem(self, rhs: Self) -> Self::Output {
        Self(self.0.rem(rhs.0), self.1.rem(rhs.1))
    }
}

impl<T> std::fmt::Debug for Point<T> where T: std::fmt::Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Point").field(&self.0).field(&self.1).finish()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub struct Rect<T>(pub Point<T>, pub Point<T>);

impl<T> Rect<T> where T: Ord {
    pub fn contains_point_inclusive(&self, p: &Point<T>) -> bool {
        p.0 >= self.0.0 && p.0 <= self.1.0
            && p.1 >= self.0.1 && p.1 <= self.1.1
    }

    pub fn contains_point(&self, p: &Point<T>) -> bool {
        p.0 >= self.0.0 && p.0 < self.1.0
            && p.1 >= self.0.1 && p.1 < self.1.1
    }
}

struct ManhattanDiamond<T> {
    current: Point<T>,
    distance: T,
    round: usize,
    remaining: T,
    zero: T,
}

impl<T> Iterator for ManhattanDiamond<T> where T: Copy + Eq + Sub<Output=T> + AddAssign + SubAssign + One {
    type Item = Point<T>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.current;
        match self.round {
            0 => {
                self.current.0 += T::one();
                self.current.1 += T::one();
            }
            1 => {
                self.current.0 -= T::one();
                self.current.1 += T::one();
            }
            2 => {
                self.current.0 -= T::one();
                self.current.1 -= T::one();
            }
            3 => {
                self.current.0 += T::one();
                self.current.1 -= T::one();
            }
            _ => {
                return None;
            }
        }

        if self.remaining == self.zero {
            self.round += 1;
            self.remaining = self.distance;
        }

        self.remaining -= T::one();

        Some(current)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone, Default)]
pub struct Vertex<T> (pub T, pub T, pub T);

impl<T> Display for Vertex<T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{},{},{}", self.0, self.1, self.2)
    }
}

impl<T> Into<Point<T>> for Vertex<T> {
    fn into(self) -> Point<T> {
        Point(self.0, self.1)
    }
}

impl<T> Vertex<T> {
    #[inline]
    pub fn x(&self) -> &T { &self.0 }
    #[inline]
    pub fn y(&self) -> &T { &self.1 }
    #[inline]
    pub fn z(&self) -> &T { &self.2 }

    #[inline]
    pub fn coord(&self, i: usize) -> &T {
        match i {
            0 => &self.0,
            1 => &self.1,
            2 => &self.2,
            _ => panic!("invalid coord index {}", i)
        }
    }

    #[inline]
    pub fn coord_mut(&mut self, i: usize) -> &mut T {
        match i {
            0 => &mut self.0,
            1 => &mut self.1,
            2 => &mut self.2,
            _ => panic!("invalid coord index {}", i)
        }
    }
}

impl<'i, T> Vertex<T> where T: 'i {
    #[inline]
    pub fn comma_separated_parser<TP>(term: TP) -> impl Parser<'i, Self> where TP: Parser<'i, T> + Copy {
        term.and_discard(b',')
            .skip_every(b' ')
            .and(term)
            .and_discard(b',')
            .skip_every(b' ')
            .and(term)
            .map(|((x, y), z)| {
                Vertex(x, y, z)
            })
    }
}

impl<T> Vertex<T> where T: Roots + Sub<Output=T> + Copy {
    pub fn distance(&self, rhs: &Vertex<T>) -> T {
        sqrt(
            pow(rhs.0 - self.0, 2)
                + pow(rhs.1 - self.1, 2)
                + pow(rhs.2 - self.2, 2)
        )
    }
}

impl<T> Vertex<T> where T: Ord {
    pub fn side_of(&self, center: &Vertex<T>) -> usize {
        (if self.x() < center.x() { 0 } else { 1 }
            | if self.y() < center.y() { 0 } else { 2 }
            | if self.z() < center.z() { 0 } else { 4 })
    }

    pub fn inside_cube(&self, cube: &Cube<T>) -> bool {
        let Cube(min, max) = cube;

        self.x() >= min.x() && self.y() >= min.y() && self.z() >= min.z()
            && self.x() < max.x() && self.y() < max.y() && self.z() < max.z()
    }
}


impl<T> Vertex<T> where T: Copy + Add<Output=T> + Sub<Output=T> {
    #[inline]
    pub fn cardinals_offset(&self, off: T) -> [Vertex<T>; 6] {
        [
            Vertex(self.0, self.1, self.2 - off),
            Vertex(self.0, self.1 - off, self.2),
            Vertex(self.0 - off, self.1, self.2),
            Vertex(self.0 + off, self.1, self.2),
            Vertex(self.0, self.1 + off, self.2),
            Vertex(self.0, self.1, self.2 + off),
        ]
    }
}

impl<T> Vertex<T> where T: One + Copy + Add<Output=T> + Sub<Output=T> {
    #[inline]
    pub fn cardinals(&self) -> [Vertex<T>; 6] {
        self.cardinals_offset(T::one())
    }
}

impl<T> Vertex<T> where T: Ord + Copy + Mul<Output=T> + Add<Output=T> + Shr<Output=T> + Zero + One + Neg<Output=T> {
    pub fn sub_center(&self, index: usize, factor: T) -> Self {
        let offset = if factor > T::one() {
            let sub_factor = factor >> T::one();
            Self::sub_center_factors(index) * Vertex(sub_factor, sub_factor, sub_factor)
        } else {
            Self::sub_center_factors2(index)
        };

        *self + offset
    }

    fn sub_center_factors(index: usize) -> Vertex<T> {
        match index {
            0 => Vertex(-T::one(), -T::one(), -T::one()),
            1 => Vertex(T::one(), -T::one(), -T::one()),
            2 => Vertex(-T::one(), T::one(), -T::one()),
            3 => Vertex(T::one(), T::one(), -T::one()),
            4 => Vertex(-T::one(), -T::one(), T::one()),
            5 => Vertex(T::one(), -T::one(), T::one()),
            6 => Vertex(-T::one(), T::one(), T::one()),
            7 => Vertex(T::one(), T::one(), T::one()),
            _ => panic!("sub_center({}) not allowed: index > 7", index),
        }
    }

    fn sub_center_factors2(index: usize) -> Vertex<T> {
        match index {
            0 => Vertex(-T::one(), -T::one(), -T::one()),
            1 => Vertex(T::zero(), -T::one(), -T::one()),
            2 => Vertex(-T::one(), T::zero(), -T::one()),
            3 => Vertex(T::zero(), T::zero(), -T::one()),
            4 => Vertex(-T::one(), -T::one(), T::zero()),
            5 => Vertex(T::zero(), -T::one(), T::zero()),
            6 => Vertex(-T::one(), T::zero(), T::zero()),
            7 => Vertex(T::zero(), T::zero(), T::zero()),
            _ => panic!("sub_center2({}) not allowed: index > 7", index),
        }
    }
}

impl<T> Neg for Vertex<T> where T: Neg<Output=T> {
    type Output = Vertex<T>;

    fn neg(self) -> Self::Output {
        Vertex(self.0.neg(), self.1.neg(), self.2.neg())
    }
}

impl<T> Mul for Vertex<T> where T: Mul<Output=T> {
    type Output = Vertex<T>;

    fn mul(self, rhs: Self) -> Self::Output {
        Vertex(self.0 * rhs.0, self.1 * rhs.1, self.2 * rhs.2)
    }
}

impl<T> Add for Vertex<T> where T: Add<Output=T> {
    type Output = Vertex<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Vertex(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl<T> Sub for Vertex<T> where T: Sub<Output=T> {
    type Output = Vertex<T>;

    fn sub(self, rhs: Self) -> Self::Output {
        Vertex(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Cube<T> (Vertex<T>, Vertex<T>);

impl<T> Cube<T> {
    #[inline]
    pub fn min(&self) -> &Vertex<T> { &self.0 }
    #[inline]
    pub fn max(&self) -> &Vertex<T> { &self.1 }
}

impl<T> Cube<T> where T: Copy + Sub<Output=T> + Mul<Output=T> {
    #[inline]
    pub fn volume(&self) -> T {
        let diff = self.1 - self.0;
        diff.0 * diff.1 * diff.2
    }
}

impl<T> Cube<T> where T: Ord {
    #[inline]
    pub fn overlaps(&self, other: &Cube<T>) -> bool {
        (self.max().x() > other.min().x())
            && (self.min().x() < other.max().x())
            && (self.max().y() > other.min().y())
            && (self.min().y() < other.max().y())
            && (self.max().z() > other.min().z())
            && (self.min().z() < other.max().z())
    }
}

impl<T> Cube<T> where T: Copy + Ord + Sub<Output=T> + One {
    #[inline]
    pub fn contains(&self, other: &Cube<T>) -> bool {
        other.contained_by(self)
    }

    #[inline]
    pub fn contained_by(&self, other: &Cube<T>) -> bool {
        let one = Vertex(T::one(), T::one(), T::one());
        self.0.inside_cube(other) && (self.1 - one).inside_cube(other)
    }
}

impl<T> Cube<T> where T: Eq + Copy + Clone + Ord {
    /// subtract returns up to 9 cubes that may result from subtracting `other` with this one.
    pub fn subtract(self, other: &Self) -> ArrayVec<Self, 9> {
        if self.overlaps(other) {
            let mut list = self.split_by(other);
            list.pop();
            list
        } else {
            self.to_vec9()
        }
    }

    fn split_by(self, other: &Self) -> ArrayVec<Self, 9> {
        if self.overlaps(other) {
            let mut cubes = ArrayVec::new();
            let mut middle = self;
            for i in 0..3 {
                let slices = middle.slash_twice(i, *other.min().coord(i), *other.max().coord(i));
                for slice in slices.iter().skip(1) {
                    if !slice.is_flat() {
                        cubes.push(*slice);
                    }
                }
                middle = slices[0];
            }

            if !middle.is_flat() {
                cubes.push(middle);
            }

            cubes
        } else {
            self.to_vec9()
        }
    }

    // slash_twice returns 1-3 cubes. The first will always be the "middle" cube.
    fn slash_twice(&self, i: usize, c1: T, c2: T) -> ArrayVec<Self, 3> {
        let (a, b) = self.slash(i, c1);
        if let Some(b) = b {
            let (b, c) = b.slash(i, c2);
            if let Some(c) = c {
                let mut v = ArrayVec::new();
                v.push(b);
                v.push(a);
                v.push(c);
                v
            } else {
                let mut v = ArrayVec::new();
                v.push(b);
                v.push(a);
                v
            }
        } else {
            let (b, c) = a.slash(i, c2);
            if let Some(c) = c {
                let mut v = ArrayVec::new();
                v.push(b);
                v.push(c);
                v
            } else {
                self.to_vec3()
            }
        }
    }

    fn slash(&self, i: usize, c: T) -> (Self, Option<Self>) {
        if c <= *self.min().coord(i) || c >= *self.max().coord(i) {
            (*self, None)
        } else {
            let mut a = *self;
            let mut b = *self;

            *a.1.coord_mut(i) = c;
            *b.0.coord_mut(i) = c;

            (a, Some(b))
        }
    }


    fn is_flat(&self) -> bool {
        for i in 0..3 {
            if self.min().coord(i) == self.max().coord(i) {
                return true;
            }
        }

        false
    }

    fn to_vec9(&self) -> ArrayVec<Self, 9> {
        let mut list = ArrayVec::new();
        list.push(*self);
        list
    }

    fn to_vec3(&self) -> ArrayVec<Self, 3> {
        let mut list = ArrayVec::new();
        list.push(*self);
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slash_conserves_volume() {
        let cube = Cube::<i32>(
            Vertex(-14, -13, -12),
            Vertex(17, 33, 23),
        );

        let target = cube.volume();

        for z in -30..30 {
            let (cube, cube2) = cube.slash(2, z);
            if let Some(cube2) = cube2 {
                assert_eq!(cube2.volume() + cube.volume(), target);
            } else {
                assert_eq!(cube.volume(), target);
            }
        }
    }

    #[test]
    fn slash_twice_adjacent() {
        let c1 = Cube::<i32>(
            Vertex(10, 10, 10),
            Vertex(15, 15, 15),
        );

        let split = c1.slash_twice(0, 15, 32);
        assert_eq!(split.as_slice(), [
            Cube::<i32>(
                Vertex(10, 10, 10),
                Vertex(15, 15, 15),
            )
        ].as_slice());

        let split = c1.slash_twice(0, 0, 10);
        assert_eq!(split.as_slice(), [
            Cube::<i32>(
                Vertex(10, 10, 10),
                Vertex(15, 15, 15),
            )
        ].as_slice());
    }

    #[test]
    fn subtract_once_works() {
        let c1 = Cube::<i32>(
            Vertex(10, 10, 10),
            Vertex(13, 13, 13),
        );
        let c2 = Cube::<i32>(
            Vertex(11, 11, 11),
            Vertex(14, 14, 14),
        );

        for c in c1.subtract(&c2) {
            println!("{:?} {}", c, c.volume());
        }

        assert_eq!(
            c1.subtract(&c2).iter()
                .map(|v| v.volume())
                .sum::<i32>(),
            19,
        );
    }

    #[test]
    fn subtract_corner() {
        let c1 = Cube::<i32>(
            Vertex(11, 11, 11),
            Vertex(14, 14, 14),
        );
        let c2 = Cube::<i32>(
            Vertex(9, 9, 9),
            Vertex(12, 12, 12),
        );

        println!("{:?}", c1.split_by(&c2));
        println!("{:?}", c2.split_by(&c1));

        assert_eq!(c1.split_by(&c2).len(), 4);
    }

    #[test]
    fn manhattan_diamond() {
        fn p(x: i32, y: i32) -> Point<i32> { Point(x, y) }

        assert_eq!(
            Point(0i32, 0i32).manhattan_diamond(4).collect::<Vec<Point<i32>>>(),
            vec![
                p(0, -4), p(1, -3), p(2, -2), p(3, -1),
                p(4, 0), p(3, 1), p(2, 2), p(1, 3),
                p(0, 4), p(-1, 3), p(-2, 2), p(-3, 1),
                p(-4, 0), p(-3, -1), p(-2, -2), p(-1, -3),
            ],
        );

        assert_eq!(
            Point(0i32, 0i32).manhattan_diamond(5).collect::<Vec<Point<i32>>>(),
            vec![
                p(0, -5), p(1, -4), p(2, -3), p(3, -2), p(4, -1),
                p(5, 0), p(4, 1), p(3, 2), p(2, 3), p(1, 4),
                p(0, 5), p(-1, 4), p(-2, 3), p(-3, 2), p(-4, 1),
                p(-5, 0), p(-4, -1), p(-3, -2), p(-2, -3), p(-1, -4),
            ],
        );
    }
}

