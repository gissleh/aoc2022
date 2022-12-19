use std::cmp::{max, Ordering};
use rayon::prelude::*;
use common::aoc::Day;
use common::parse3::{Parser, unsigned_int};
use common::search2;
use common::search2::Search;

pub fn main(day: &mut Day, input: &[u8]) {
    let blueprints = day.run_parse(1000, || parse(input));

    day.run(1, "", 1, || part1(&blueprints));
    day.run(2, "", 1, || part2(&blueprints));
}

fn parse(data: &[u8]) -> Vec<Blueprint> {
    Blueprint::parser().skip(b'\n').repeat().parse(data).unwrap()
}

fn part1(blueprints: &[Blueprint]) -> u32 {
    blueprints.par_iter()
        .map(|b| b.index as u32 * (b.maximize(25) as u32))
        .sum()
}

fn part2(blueprints: &[Blueprint]) -> u32 {
    blueprints.par_iter()
        .take(3)
        .map(|b| b.maximize(33) as u32)
        .product()
}

#[derive(Eq, PartialEq, Debug)]
struct Blueprint {
    index: u8,
    orc: u16,
    cc: u16,
    obc: [u16; 2],
    gc: [u16; 2],
}

impl Blueprint {
    fn maximize(&self, minutes: u8) -> u16 {
        let mut max_geodes = [0u16; 64];
        let robo_limits = [
            max(max(self.orc, self.cc), max(self.gc[0], self.obc[0])),
            self.obc[1],
            self.gc[1],
        ];

        search2::bfs(State::new())
            .run(|search: _, s: &State| {
                if s.minute == minutes {
                    return Some(s.resources[3]);
                }

                let geode_score = s.resources[3];
                let max_geodes = max_geodes.get_mut(s.minute as usize - 1).unwrap();
                if geode_score >= *max_geodes {
                    *max_geodes = geode_score;
                } else {
                    return None;
                }

                let next = s.next();
                let mut add_next = true;
                if s.resources[0] >= self.gc[0]
                    && s.resources[2] >= self.gc[1] {
                    search.add_step(next.clone()
                        .costing(0, self.gc[0])
                        .costing(2, self.gc[1])
                        .having(3));

                    add_next = false;
                }
                if s.robots[2] < robo_limits[2] && s.resources[0] >= self.obc[0]
                    && s.resources[1] >= self.obc[1] {
                    search.add_step(next.clone()
                        .costing(0, self.obc[0])
                        .costing(1, self.obc[1])
                        .having(2));

                    add_next = false;
                }

                if add_next {
                    if s.robots[0] < robo_limits[0] && s.resources[0] >= self.orc {
                        search.add_step(next.clone()
                            .costing(0, self.orc)
                            .having(0));
                    }
                    if s.robots[1] < robo_limits[1] && s.resources[0] >= self.cc {
                        search.add_step(next.clone()
                            .costing(0, self.cc)
                            .having(1));
                    }
                }



                    search.add_step(next);

                None
            })
            .max().unwrap()
    }

    fn parser<'i>() -> impl Parser<'i, Blueprint> {
        b"Blueprint ".and_instead(unsigned_int::<u8>())
            .and_discard(b": Each ore robot costs ")
            .and(unsigned_int::<u16>())
            .and_discard(b" ore. Each clay robot costs ")
            .and(unsigned_int::<u16>())
            .and_discard(b" ore. Each obsidian robot costs ")
            .and(unsigned_int::<u16>())
            .and_discard(b" ore and ")
            .and(unsigned_int::<u16>())
            .and_discard(b" clay. Each geode robot costs ")
            .and(unsigned_int::<u16>())
            .and_discard(b" ore and ")
            .and(unsigned_int::<u16>())
            .and_discard(b" obsidian.")
            .map(|((((((index, orc), cc), obc_ore), obc_clay), gc_ore), gc_obsidian)| {
                Blueprint {
                    index,
                    orc,
                    cc,
                    obc: [obc_ore, obc_clay],
                    gc: [gc_ore, gc_obsidian],
                }
            })
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy, Default, Debug)]
struct State {
    minute: u8,
    robots: [u16; 4],
    resources: [u16; 4],
}

impl State {
    #[allow(dead_code)]
    fn robot_cmp(&self, other: &Self) -> Ordering {
        (self.robots[3]).cmp(&(other.robots[3]))
            .then_with(|| self.resources[3].cmp(&other.resources[3]))
            .then_with(|| self.robots[2].cmp(&other.robots[2]))
            .then_with(|| self.robots[1].cmp(&other.robots[1]))
            .then_with(|| self.robots[0].cmp(&other.robots[0]))
    }

    fn next(&self) -> Self {
        let mut next = self.clone();
        for i in 0..4 {
            next.resources[i] += next.robots[i];
        }
        next.minute += 1;
        next
    }

    fn costing(mut self, i: usize, n: u16) -> Self {
        self.resources[i] -= n;
        self
    }

    fn having(mut self, i: usize) -> Self {
        self.robots[i] += 1;
        self
    }

    fn new() -> Self {
        State {
            minute: 1,
            robots: [1, 0, 0, 0],
            resources: [0, 0, 0, 0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = include_bytes!("./test_fixtures/d19_p1_example.txt");
    const P1_EXAMPLE_BLUEPRINTS: &[Blueprint] = &[
        Blueprint {
            index: 1,
            orc: 4,
            cc: 2,
            obc: [3, 14],
            gc: [2, 7],
        },
        Blueprint {
            index: 2,
            orc: 2,
            cc: 3,
            obc: [3, 8],
            gc: [3, 12],
        },
    ];

    #[test]
    fn parse_works() {
        assert_eq!(parse(P1_EXAMPLE).as_slice(), P1_EXAMPLE_BLUEPRINTS);
    }

    #[test]
    fn blueprint_maximize_maximizes_maximally() {
        assert_eq!(P1_EXAMPLE_BLUEPRINTS[0].maximize(25), 9);
        assert_eq!(P1_EXAMPLE_BLUEPRINTS[1].maximize(25), 12);
    }
}