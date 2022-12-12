use std::cmp::max;
use crate::geo::Point;

pub struct QuadTree<T> {
    default_value: T,
    root: usize,
    root_size: isize,
    branches: Vec<Quadrant<T>>,
    free_list: Vec<usize>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Quadrant<T> {
    Leaf(T),
    Branch([usize; 4]),
}

impl<T> Default for QuadTree<T> where T: Default + Copy {
    fn default() -> Self {
        Self::new(1, T::default())
    }
}

impl<T> QuadTree<T> where T: Default + Copy {
    pub fn default_with_levels(levels: u32) -> Self {
        Self::new(levels, T::default())
    }
}

impl<T> QuadTree<T> where T: Copy {
    pub fn new(levels: u32, default_value: T) -> Self {
        Self {
            default_value,

            root_size: 1 << (levels - 1),
            branches: vec![Quadrant::Leaf(default_value)],
            root: 0,
            free_list: Vec::new(),
        }
    }

    pub fn get_mut(&mut self, pos: &Point<isize>) -> Option<&mut T> {
        let drill_index = self.drill(&pos);
        if let Quadrant::Leaf(v) = &mut self.branches[drill_index] {
            Some(v)
        } else {
            panic!("Drill got ")
        }
    }

    #[allow(dead_code)]
    fn from_quadrants_unchecked(default_value: T, levels: u32, s: &[Quadrant<T>]) -> Self {
        Self {
            default_value,
            root_size: 1 << (levels - 1),
            free_list: Vec::new(),
            root: 0,
            branches: s.iter().copied().collect(),
        }
    }

    #[inline]
    fn drill(&mut self, target: &Point<isize>) -> usize {
        while !self.envelops_point(target) {
            self.grow_out();
        }

        self.drill_step(self.root, Point(0, 0), self.root_size, target)
    }

    #[inline]
    fn drill_step(&mut self, index: usize, center: Point<isize>, distance: isize, target: &Point<isize>) -> usize {
        #[cfg(test)] println!("drill_step({}, {:?}, {:?}, {:?})", index, center, distance, target);

        match self.branches[index] {
            Quadrant::Leaf(_) => {
                if distance == 0 {
                    #[cfg(test)] println!("  found: {}", index);
                    index
                } else {
                    self.force_branch(index);
                    self.drill_step(index, center, distance, target)
                }
            }
            Quadrant::Branch(children) => {
                #[cfg(test)] assert!(distance > 0);

                let side = target.side_of(&center);
                let new_distance = distance / 2;
                let new_center = center.move_center(side, max(new_distance, 1));

                self.drill_step(children[side], new_center, new_distance, target)
            }
        }
    }

    fn grow_out(&mut self) {
        if let Quadrant::Branch(root) = self.branches[self.root] {
            let mut leaves = [0usize; 12];
            for i in 0..12 {
                leaves[i] = self.add_quadrant(Quadrant::Leaf(self.default_value));
            }

            self.branches[self.root] = Quadrant::Branch([
                self.add_quadrant(Quadrant::Branch([leaves[0], leaves[1], leaves[2], root[0]])),
                self.add_quadrant(Quadrant::Branch([leaves[3], leaves[4], root[1], leaves[5]])),
                self.add_quadrant(Quadrant::Branch([leaves[6], root[2], leaves[7], leaves[8]])),
                self.add_quadrant(Quadrant::Branch([root[3], leaves[9], leaves[10], leaves[11]])),
            ]);
        }

        self.root_size <<= 1;
    }

    fn add_quadrant(&mut self, branch: Quadrant<T>) -> usize {
        if let Some(index) = self.free_list.pop() {
            self.branches[index] = branch;
            index
        } else {
            self.branches.push(branch);
            self.branches.len() - 1
        }
    }

    fn force_branch(&mut self, index: usize) {
        if let Quadrant::Leaf(t) = self.branches[index] {
            self.branches[index] = Quadrant::Branch([
                self.add_quadrant(Quadrant::Leaf(t)),
                self.add_quadrant(Quadrant::Leaf(t)),
                self.add_quadrant(Quadrant::Leaf(t)),
                self.add_quadrant(Quadrant::Leaf(t)),
            ]);
        }
    }
}

impl<T> QuadTree<T> where T: Eq {
    pub fn compact(&mut self) {
        self.compact_branch(self.root, self.root, 0);
    }

    fn compact_branch(&mut self, index: usize, parent_index: usize, parent_pos: usize) {
        if let Quadrant::Branch(children) = self.branches[index] {
            // Propagate down, then stop if there's still sub-branches
            for (pos, sub_index) in children.iter().enumerate() {
                self.compact_branch(*sub_index, index, pos);

                if let Quadrant::Branch(_) = &self.branches[*sub_index] {
                    return;
                }
            }

            // Stop if the branches aren't equal
            for index in children.iter().skip(1) {
                if self.branches[children[0]] != self.branches[*index] {
                    return;
                }
            }

            // Delete
            let new_index = children[0];
            self.free_list.push(index);
            for index in children.iter().skip(1) {
                self.free_list.push(*index);
            }

            if parent_index == index {
                self.root = new_index;
            } else {
                if let Quadrant::Branch(children) = &mut self.branches[parent_index] {
                    children[parent_pos] = new_index
                } else {
                    panic!("Parent is leaf");
                }
            }
        }
    }
}

impl<T> QuadTree<T> {
    pub fn get(&self, pos: &Point<isize>) -> Option<&T> {
        if let Some((index, _)) = self.seek(pos) {
            if let Quadrant::Leaf(v) = &self.branches[index] {
                Some(v)
            } else {
                panic!("QuadTree::get should not get a branch")
            }
        } else {
            None
        }
    }

    pub fn quadrants(&self) -> usize {
        self.branches.len() - self.free_list.len()
    }

    fn seek(&self, target: &Point<isize>) -> Option<(usize, isize)> {
        if self.envelops_point(target) {
            Some(self.seek_step(self.root, Point(0, 0), self.root_size, target))
        } else {
            None
        }
    }

    fn seek_step(&self, index: usize, center: Point<isize>, distance: isize, target: &Point<isize>) -> (usize, isize) {
        #[cfg(test)] println!("seek_step({}, {:?}, {:?}, {:?})", index, center, distance, target);

        match &self.branches[index] {
            Quadrant::Leaf(_) => {
                #[cfg(test)] println!("  found: {}, {}", index, distance);

                (index, distance)
            }
            Quadrant::Branch(children) => {
                #[cfg(test)] assert!(distance > 0);

                let side = target.side_of(&center);
                let new_distance = distance / 2;
                let new_center = center.move_center(side, max(new_distance, 1));

                self.seek_step(children[side], new_center, new_distance, target)
            }
        }
    }

    fn envelops_point(&self, p: &Point<isize>) -> bool {
        p.0 >= -self.root_size && p.1 >= -self.root_size && p.1 < self.root_size && p.1 < self.root_size
    }
}

#[cfg(test)]
mod tests {
    use crate::quadtree::Quadrant::{Branch, Leaf};
    use crate::wfc::WFCBehavior::Quadrant;
    use super::*;

    fn assert_tree<const N: usize, T: Eq + std::fmt::Debug>(tree: &QuadTree<T>, expected: [[T; N]; N]) {
        let half = N as isize / 2;

        for y in 0..N {
            for x in 0..N {
                let p = &Point(x as isize - half, y as isize - half);

                assert_eq!(
                    tree.get(p),
                    Some(&expected[y as usize][x as usize]),
                    "{:?}", p,
                );
            }
        }
    }

    #[test]
    fn test_grow() {
        let mut tree = QuadTree::from_quadrants_unchecked(0u32, 1, &[
            Branch([1, 2, 3, 4]),
            Leaf(1), Leaf(2), Leaf(3), Leaf(4),
        ]);

        assert_eq!(tree.root_size, 1);
        assert_tree(&tree, [
            [1, 2],
            [3, 4],
        ]);

        tree.grow_out();

        assert_eq!(tree.root_size, 2);
        assert_eq!(tree.branches, vec![
            Branch([17, 18, 19, 20]),
            Leaf(1), Leaf(2), Leaf(3), Leaf(4),
            Leaf(0), Leaf(0), Leaf(0), Leaf(0),
            Leaf(0), Leaf(0), Leaf(0), Leaf(0),
            Leaf(0), Leaf(0), Leaf(0), Leaf(0),
            Branch([5, 6, 7, 1]),
            Branch([8, 9, 2, 10]),
            Branch([11, 3, 12, 13]),
            Branch([4, 14, 15, 16]),
        ]);
        assert_tree(&tree, [
            [0, 0, 0, 0],
            [0, 1, 2, 0],
            [0, 3, 4, 0],
            [0, 0, 0, 0],
        ]);

        tree.grow_out();
        assert_tree(&tree, [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 1, 2, 0, 0, 0],
            [0, 0, 0, 3, 4, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ]);
        assert_eq!(tree.branches.len(), 37);

        let mut tree = QuadTree::new(1, 1u32);
        tree.grow_out();
        assert_eq!(tree.root_size, 2);
        assert_eq!(tree.branches.len(), 1);
        tree.grow_out();
        assert_eq!(tree.root_size, 4);
        assert_eq!(tree.branches.len(), 1);
    }

    #[test]
    fn test_get() {
        let tree = QuadTree::from_quadrants_unchecked(0u32, 3, &[
            Branch([1, 2, 3, 4]),
            Leaf(0), Leaf(1), Leaf(2), Branch([5, 6, 7, 8]),
            Leaf(4), Branch([9, 10, 11, 12]), Leaf(6), Leaf(7),
            Leaf(5), Leaf(8), Leaf(5), Leaf(5),
        ]);

        assert_tree(&tree, [
            [0, 0, 0, 0, 1, 1, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [2, 2, 2, 2, 4, 4, 5, 8],
            [2, 2, 2, 2, 4, 4, 5, 5],
            [2, 2, 2, 2, 6, 6, 7, 7],
            [2, 2, 2, 2, 6, 6, 7, 7],
        ]);

        let tree = QuadTree::from_quadrants_unchecked(0u32, 2, &[
            Branch([1, 2, 3, 4]),
            Branch([5, 6, 7, 8]),
            Branch([9, 10, 11, 12]),
            Branch([13, 14, 15, 16]),
            Branch([17, 18, 19, 20]),
            Leaf(1), Leaf(2), Leaf(5), Leaf(6),
            Leaf(3), Leaf(4), Leaf(7), Leaf(8),
            Leaf(9), Leaf(10), Leaf(13), Leaf(14),
            Leaf(11), Leaf(12), Leaf(15), Leaf(16),
        ]);

        assert_tree(&tree, [
            [1, 2, 3, 4],
            [5, 6, 7, 8],
            [9, 10, 11, 12],
            [13, 14, 15, 16],
        ]);
    }

    #[test]
    fn test_force_branch() {
        let mut tree = QuadTree::from_quadrants_unchecked(0u32, 2, &[
            Branch([1, 2, 3, 4]),
            Leaf(1), Leaf(2), Leaf(3), Leaf(4),
        ]);

        tree.force_branch(2);

        assert_eq!(tree.branches, vec![
            Branch([1, 2, 3, 4]),
            Leaf(1), Branch([5, 6, 7, 8]), Leaf(3), Leaf(4),
            Leaf(2), Leaf(2), Leaf(2), Leaf(2),
        ]);
    }

    #[test]
    fn test_drill() {
        let leaf = Leaf(1u32);

        let mut tree = QuadTree::new(4, 1u32);
        assert_eq!(tree.drill(&Point(-1, -1)), 16);
        assert_eq!(tree.branches, vec![
            Branch([1, 2, 3, 4]), Branch([5, 6, 7, 8]), leaf, leaf, leaf,
            leaf, leaf, leaf, Branch([9, 10, 11, 12]), leaf, leaf, leaf,
            Branch([13, 14, 15, 16]), leaf, leaf, leaf, leaf,
        ])
    }
}
