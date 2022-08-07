use crate::geo::Vertex;

pub struct Octree<T> {
    root: Octant<T>,
    factor: isize,
    max_depth: usize,
}

impl<T> Octree<T> {
    pub fn get(&self, v: &Vertex<isize>) -> Option<&T> {
        self.root.get(v, &Vertex(0, 0, 0), self.factor)
    }
}

impl<T> Octree<T> where T: Copy {
    pub fn set(&mut self, v: &Vertex<isize>, value: Option<T>) {
        self.root.set(v, &Vertex(0, 0, 0), self.factor, 0, self.max_depth, value)
    }
}

pub enum Octant<T> {
    Leaf(Option<T>),
    Branch(Box<[Octant<T>; 8]>),
}

impl<T> Octant<T> {
    fn get(&self, v: &Vertex<isize>, c: &Vertex<isize>, factor: isize) -> Option<&T> {
        match self {
            Octant::Leaf(value) => value.into(),
            Octant::Branch(subs) => {
                let i = v.side_of(c);
                let sub_center = c.sub_center(i, factor);

                subs[i].get(v, &sub_center, factor >> 1)
            }
        }
    }
}

impl<T> Octant<T> where T: Copy {
    fn set(&mut self, v: &Vertex<isize>, c: &Vertex<isize>, factor: isize, depth: usize, max_depth: usize, value: Option<T>) {
        if factor == 0 || (max_depth != 0 && depth == max_depth) {
            *self = Octant::Leaf(value)
        }

        if let Octant::Leaf(leaf_value) = self {
            *self = Octant::Branch(Box::new([
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
                Octant::Leaf(*leaf_value),
            ]))
        }

        if let Octant::Branch(subs) = self {
            let i = v.side_of(c);
            let sub_center = c.sub_center(i, factor);

            subs[i].set(v, &sub_center, factor >> 1, depth + 1, max_depth, value);
        }
    }
}

impl<T> Clone for Octant<T> where T: Clone {
    fn clone(&self) -> Self {
        match self {
            Octant::Leaf(value) => Octant::Leaf(value.clone()),
            Octant::Branch(subs) => Octant::Branch(subs.clone()),
        }
    }
}
