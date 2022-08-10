use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::mem;
use rustc_hash::FxHashSet;

common::day!(parse, part1, part2, 100000, 1000, 50);

#[derive(Hash, Copy, Clone, Eq, PartialEq, Debug)]
struct Bugs(u32);

impl Display for Bugs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Bugs {
    fn to_string(&self) -> String {
        let mut s = String::with_capacity(64);

        for i in 0..25 {
            if i == 12 {
                s.push('?');
            } else {
                let has_bug = (self.0 >> i) & 1 == 1;
                if has_bug {
                    s.push('#');
                } else {
                    s.push('.');
                }
            }
            if i % 5 == 4 {
                s.push('\n');
            }
        }

        s
    }

    fn next_minute(&self) -> Bugs {
        let mut next = 0u32;

        for i in 0..25 {
            let has_bug = (self.0 >> i) & 1 == 1;
            let mut adj_bugs = 0;
            let x = i % 5;
            let y = i / 5;

            if x > 0 { adj_bugs += (self.0 >> (i - 1)) & 1; }
            if x < 4 { adj_bugs += (self.0 >> (i + 1)) & 1; }
            if y > 0 { adj_bugs += (self.0 >> (i - 5)) & 1; }
            if y < 4 { adj_bugs += (self.0 >> (i + 5)) & 1; }

            let can_grow = if has_bug { adj_bugs == 1 } else { adj_bugs == 1 || adj_bugs == 2 };
            if can_grow {
                next |= 1 << i;
            }
        }

        Bugs(next)
    }

    fn next_minute_recursive(&self, outer: &Bugs, inner: &Bugs) -> Bugs {
        let mut next = 0u32;

        for i in 0..25 {
            if i == 12 {
                continue;
            }

            let has_bug = (self.0 >> i) & 1 == 1;
            let mut adj_bugs = 0;
            let x = i % 5;
            let y = i / 5;

            if x > 0 { adj_bugs += (self.0 >> (i - 1)) & 1; }
            if x < 4 { adj_bugs += (self.0 >> (i + 1)) & 1; }
            if y > 0 { adj_bugs += (self.0 >> (i - 5)) & 1; }
            if y < 4 { adj_bugs += (self.0 >> (i + 5)) & 1; }

            if x == 0 { adj_bugs += (outer.0 >> 11) & 1; }
            if x == 4 { adj_bugs += (outer.0 >> 13) & 1; }
            if y == 0 { adj_bugs += (outer.0 >> 7) & 1; }
            if y == 4 { adj_bugs += (outer.0 >> 17) & 1; }

            if x == 2 && y == 1 { adj_bugs += (inner.0 & 0b00000_00000_00000_00000_11111).count_ones() }
            else if x == 2 && y == 3 { adj_bugs += (inner.0 & 0b11111_00000_00000_00000_00000).count_ones() }
            else if x == 1 && y == 2 { adj_bugs += (inner.0 & 0b00001_00001_00001_00001_00001).count_ones() }
            else if x == 3 && y == 2 { adj_bugs += (inner.0 & 0b10000_10000_10000_10000_10000).count_ones() }

            let can_grow = if has_bug { adj_bugs == 1 } else { adj_bugs == 1 || adj_bugs == 2 };
            if can_grow {
                next |= 1 << i;
            }
        }

        Bugs(next)
    }

    fn parse(data: &[u8]) -> Bugs {
        let mut res = 0u32;
        let mut mask = 1u32;

        for v in data {
            match *v {
                b'#' => {
                    res = res | mask;
                    mask <<= 1;
                }
                b'.' => { mask <<= 1; }
                b'\n' | b' ' => {}
                _ => unreachable!(),
            }
        }

        Bugs(res)
    }
}

fn part1(input: &Bugs) -> u32 {
    let mut seen = FxHashSet::default();
    seen.insert(*input);

    let mut current = *input;
    loop {
        current = current.next_minute();
        if !seen.insert(current) {
            return current.0;
        }
    }
}

fn part2(input: &Bugs) -> u32 {
    run_recursive_growth(input, 200).into_iter().map(|v| v.0.count_ones()).sum()
}

fn run_recursive_growth(input: &Bugs, rounds: u32) -> VecDeque<Bugs> {
    let mut levels = VecDeque::with_capacity(512);
    levels.push_back(Bugs(0));
    levels.push_back(Bugs(0));
    levels.push_back(*input);
    levels.push_back(Bugs(0));
    levels.push_back(Bugs(0));
    let mut next_levels = levels.clone();

    for _ in 0..rounds {
        for i in 1..levels.len() - 1 {
            next_levels[i] = levels[i].next_minute_recursive(&levels[i - 1], &levels[i + 1]);
        }

        mem::swap(&mut levels, &mut next_levels);
        if levels.get(2).unwrap().0 != 0 {
            levels.push_front(Bugs(0));
            next_levels.push_front(Bugs(0));
        }
        if levels.get(levels.len() - 2).unwrap().0 != 0 {
            levels.push_back(Bugs(0));
            next_levels.push_back(Bugs(0));
        }
    }

    levels
}

fn parse(data: &[u8]) -> Bugs {
   Bugs::parse(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &'static [u8] = b"....#
#..#.
#..##
..#..
#....";

    #[test]
    fn p1_works_on_example() {
        let bugs = Bugs::parse(SAMPLE);
        let result = part1(&bugs);

        assert_eq!(result, 2129920);
    }

    #[test]
    fn p2_works_on_example() {
        let bugs = Bugs::parse(SAMPLE);
        let levels = run_recursive_growth(&bugs, 10);

        for i in (0..levels.len()).skip_while(|l| levels[*l].0 == 0) {
            println!("--- {} ---", i);
            println!("{}", levels[i]);
        }

        assert_eq!(levels.iter().map(|v| v.0.count_ones()).sum::<u32>(), 99);
    }
}