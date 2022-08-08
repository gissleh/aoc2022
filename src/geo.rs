use std::fmt::Formatter;
use std::ops::{Add, Mul, Neg, Sub, Shr};
use num::integer::{sqrt, Roots};
use num::{pow, One, Zero};

#[derive(Hash)]
pub struct Point<T> (pub T, pub T);

impl<T> Point<T> {
    #[inline]
    pub fn x(&self) -> &T { &self.0 }
    #[inline]
    pub fn y(&self) -> &T { &self.1 }
}

impl<T> Point<T> where T: Roots + Sub<Output=T> + Copy {
    pub fn distance(&self, rhs: &Point<T>) -> T {
        sqrt(
            pow(rhs.0 - self.0, 2)
                + pow(rhs.1 - self.1, 2)
        )
    }
}

impl<T> Point<T> where T: One + Copy + Add<Output=T> + Sub<Output=T> {
    pub fn cardinals(&self) -> [Point<T>; 4] {
        [
            Point(self.0, self.1 - T::one()),
            Point(self.0 - T::one(), self.1),
            Point(self.0 + T::one(), self.1),
            Point(self.0, self.1 + T::one()),
        ]
    }

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

impl<T> Add for Point<T> where T: Add<Output=T> {
    type Output = Point<T>;

    fn add(self, rhs: Self) -> Self::Output {
        Point(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl<T> PartialEq for Point<T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && self.1 == other.1
    }

    fn ne(&self, other: &Self) -> bool {
        self.0 != other.0 || self.1 != other.1
    }
}

impl<T> Eq for Point<T> where T: Eq {}

impl<T> std::fmt::Debug for Point<T> where T: std::fmt::Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Point").field(&self.0).field(&self.1).finish()
    }
}


pub struct Vertex<T> (pub T, pub T, pub T);

impl<T> Vertex<T> {
    #[inline]
    pub fn x(&self) -> &T { &self.0 }
    #[inline]
    pub fn y(&self) -> &T { &self.1 }
    #[inline]
    pub fn z(&self) -> &T { &self.2 }
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

impl<T> Copy for Vertex<T> where T: Copy {}

impl<T> Clone for Vertex<T> where T: Clone {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1.clone(), self.2.clone())
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
        (self.min().x() > other.max().x())
            && (self.min().x() < other.max().x())
            && (self.min().y() > other.max().y())
            && (self.min().y() < other.max().y())
            && (self.min().z() > other.max().z())
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

