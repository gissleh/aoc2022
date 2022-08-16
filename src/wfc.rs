use crate::geo::Point;
use crate::grid2::{FillableGrid, FixedGrid, GetterGrid, IterableSliceGrid};

pub struct WaveGrid<G> where G: FixedGrid + GetterGrid<u64> + FillableGrid<u64> + IterableSliceGrid<u64> {
    data: G,
    palette_mask: u64,
}

impl<G> WaveGrid<G> where G: FixedGrid + GetterGrid<u64> + FillableGrid<u64> + IterableSliceGrid<u64> {
    pub fn subtract(&mut self, p: &Point<usize>, index: usize) {
        if let Some(v) = self.data.get_mut(p) {
            *v &= !(1 << index) & self.palette_mask;
        }
    }

    pub fn collapse(&mut self, p: &Point<usize>, index: usize, behaviors: &[WFCBehavior]) -> bool {
        let mut found = false;

        if let Some(v) = self.data.get_mut(p) {
            let mask = 1 << index;
            if *v & mask == mask {
                *v = mask;
                found = true;
            }
        }

        if found {
            for behavior in behaviors {
                self.subtract_with(p, index, *behavior)
            }
        }

        found
    }

    pub fn first_value(&self, p: &Point<usize>) -> Option<usize> {
        self.data.get(p).map(|v| v.trailing_zeros() as usize)
    }

    pub fn values<'a, T>(&'a self, palette: &'a [T], blank: T) -> impl Iterator<Item=(Point<usize>, T)> + 'a where T: Copy + Clone {
        self.data.cells().map(move |(p, mask)| {
            if *mask == 0 {
                (p, blank)
            } else {
                (p, palette[mask.trailing_zeros() as usize].clone())
            }
        })
    }

    pub fn first_lowest_entropy(&self) -> Option<Point<usize>> {
        let mut lowest_point = None;
        let mut lowest_entropy = 65;

        for (p, v) in self.data.cells() {
            let entropy = v.count_ones();
            if entropy > 1 && entropy < lowest_entropy {
                lowest_entropy = entropy;
                lowest_point = Some(p);
            }
        }

        lowest_point
    }

    pub fn subtract_with(&mut self, p: &Point<usize>, index: usize, behavior: WFCBehavior) {
        match behavior {
            WFCBehavior::Row => {
                let Point(x, y) = *p;
                for x2 in 0..self.data.width() {
                    if x2 == x { continue; }
                    self.subtract(&Point(x2, y), index)
                }
            }
            WFCBehavior::Column => {
                let Point(x, y) = *p;
                for y2 in 0..self.data.width() {
                    if y2 == y { continue; }
                    self.subtract(&Point(x, y2), index)
                }
            }
            WFCBehavior::Quadrant(w, h) => {
                let x1 = (p.0 / w) * w;
                let y1 = (p.1 / h) * h;

                for x in x1..x1 + w {
                    for y in y1..y1 + w {
                        let p2 = Point(x, y);
                        if p.eq(&p2) {
                            continue;
                        }

                        self.subtract(&p2, index);
                    }
                }
            }
            WFCBehavior::Adjacent(mut mask) => {
                for p in p.neighbors() {
                    if mask & 1 == 1 {
                        self.subtract(&p, index);
                    }

                    mask >>= 1;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.data.fill(self.palette_mask)
    }

    pub fn new(data: G, palette_len: usize) -> Self {
        Self { data, palette_mask: (1 << palette_len) - 1 }
    }
}

#[derive(Copy, Clone)]
pub enum WFCBehavior {
    Row,
    Column,
    Quadrant(usize, usize),
    Adjacent(u8),
}

#[cfg(test)]
mod tests {
    use crate::grid2::ArrayGrid;
    use super::*;

    #[test]
    fn wavegrid_collpase_and_subtract() {
        let mut wg = WaveGrid::new(ArrayGrid::<u64, 36, 6>::new(), 9);
        wg.clear();

        assert_eq!(wg.collapse(&Point(3, 3), 1, &[]), true);

        wg.subtract(&Point(3, 2), 1);
        assert_eq!(wg.collapse(&Point(3, 2), 1, &[]), false);

        assert_eq!(*wg.data.get(&Point(3, 3)).unwrap(), 0b10);
        assert_eq!(*wg.data.get(&Point(3, 2)).unwrap(), 0b111111101);

        wg.subtract(&Point(3, 2), 4);
        assert_eq!(*wg.data.get(&Point(3, 2)).unwrap(), 0b111101101);
    }

    #[test]
    fn wavegrid_simple_sudoku() {
        const BEHAVIORS: &'static [WFCBehavior; 3] = &[WFCBehavior::Row, WFCBehavior::Column, WFCBehavior::Quadrant(2, 2)];
        const EXPECTED_OUTCOME: &'static [u8] = &[
            4, 1, 2, 3,
            2, 3, 4, 1,
            1, 4, 3, 2,
            3, 2, 1, 4,
        ];

        let mut wg = WaveGrid::new(ArrayGrid::<u64, 16, 4>::new(), 4);
        wg.clear();

        wg.collapse(&Point(0, 0), 3, BEHAVIORS);
        wg.collapse(&Point(2, 1), 3, BEHAVIORS);
        wg.collapse(&Point(1, 2), 3, BEHAVIORS);
        wg.collapse(&Point(3, 3), 3, BEHAVIORS);

        while let Some(p) = wg.first_lowest_entropy() {
            let index = wg.first_value(&p).unwrap();
            wg.collapse(&p, index, BEHAVIORS);
        }

        let v: Vec<u8> = wg.values(&[1u8, 2, 3, 4], 0).map(|(_, v)| v).collect();

        assert_eq!(v.as_slice(), EXPECTED_OUTCOME);
    }
}