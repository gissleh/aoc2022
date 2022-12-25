use std::cmp::{max, Ordering};
use std::hash::{Hash, Hasher};
use std::ops::Add;
use smallvec::SmallVec;
use common::aoc::Day;
use common::graph::Graph;
use common::parse3::{n_bytes_array, Parser, unsigned_int};
use common::search::{BFS, BFSResult, Dijkstra, DijkstraResult};

pub fn main(day: &mut Day, input: &[u8]) {
    let graph: Graph<Valve, i8> = day.run_parse(1000, || parse(input));

    day.note("Valves", graph.len());

    day.run(1, "", 100, || part1(&graph));
    day.run(1, "DFS", 100, || part1_dfs(&graph));
    //day.run(2, "", 1, || part2(&graph));
    day.run(2, "DFS", 1, || part2_dfs(&graph));
}

fn parse(data: &[u8]) -> Graph<Valve, i8> {
    // Parse and build unweighted graph
    let graph_unweighted: Graph<Valve, ()> = b"Valve ".and_instead(n_bytes_array::<2>())
        .and_discard(b" has flow rate=")
        .and(unsigned_int::<u64>())
        .and_discard(b"; tunnels lead to valves ".or(b"; tunnel leads to valve "))
        .and(n_bytes_array::<2>().repeat_delimited(b", "))
        .skip(b'\n')
        .repeat_fold_mut(
            || {
                let mut graph = Graph::new();
                graph.insert(Valve {
                    name: [b'A', b'A'],
                    flow_rate: 0,
                });

                graph
            },
            |graph: &mut Graph<Valve, ()>, ((name, flow_rate), list): (([u8; 2], u64), Vec<[u8; 2]>)| {
                let current_index = graph.find_by(|v| v.name == name)
                    .and_then(|i| {
                        graph.node_mut(i).unwrap().flow_rate = flow_rate;
                        Some(i)
                    })
                    .or_else(|| Some(graph.insert(Valve { name, flow_rate })))
                    .unwrap();

                for name in list {
                    let next_index = graph.find_by(|v| v.name == name)
                        .or_else(|| Some(graph.insert(Valve { name, flow_rate: 0 })))
                        .unwrap();

                    graph.connect_mutual(current_index, next_index, ());
                }
            })
        .parse(data).unwrap();

    // Build a weighted graph
    let mut graph_weighted: Graph<Valve, i8> = Graph::new();
    let mut bfs: BFS<usize, usize> = BFS::new();
    graph_weighted.insert(Valve { flow_rate: 0, name: [b'A', b'A'] });
    for i in 0..graph_unweighted.len() {
        let valve = graph_unweighted.node(i).unwrap();
        if valve.flow_rate > 0 || i == 0 {
            bfs.run(i, |index| {
                let valve = graph_unweighted.node(*index).unwrap();
                let next = graph_unweighted.edges(*index).unwrap()
                    .map(|(i, _)| *i)
                    .collect();

                if *index != i && (valve.flow_rate > 0 || *index == 0) {
                    return BFSResult::Found(*index, next);
                } else {
                    BFSResult::Continue(next)
                }
            });

            let current_index = graph_weighted.find_by(|v| v.name == valve.name)
                .or_else(|| Some(graph_weighted.insert(valve.clone())))
                .unwrap();

            for (index, steps) in bfs.found_goals().iter() {
                let valve = graph_unweighted.node(*index).unwrap();
                let new_index = graph_weighted.find_by(|v| v.name == valve.name)
                    .or_else(|| Some(graph_weighted.insert(valve.clone())))
                    .unwrap();

                graph_weighted.connect(current_index, new_index, *steps as i8 + 1)
            }
        }
    }

    graph_weighted
}

fn part1(graph: &Graph<Valve, i8>) -> u64 {
    let mut dijkstra: Dijkstra<State, Pressure> = Dijkstra::new(false, false);
    let initial_state = State {
        minutes: 30,
        opened: 0,
        index: 0usize,
    };
    let all_valves = (1 << graph.len()) - 1;

    let (pressure, _) = dijkstra.run(initial_state, Pressure(0), |state: &State| {
        // Time's up
        if state.minutes <= 0 || state.opened == all_valves {
            return DijkstraResult::Success;
        }

        let mut next_steps = SmallVec::new();
        let valve = graph.node(state.index).unwrap();
        let (state, new_pressure) = state.with_open(valve.flow_rate);

        for (next_index, min) in graph.edges(state.index).unwrap() {
            if state.has_opened(*next_index) {
                continue;
            }

            next_steps.push((new_pressure, Pressure(0), state.with_visit(
                *next_index, *min,
            )));
        }

        DijkstraResult::Continue(next_steps)
    }).unwrap();

    pressure.0
}

fn part1_dfs(graph: &Graph<Valve, i8>) -> u64 {
    let all_valves = (1 << graph.len()) - 1;
    let mut stack = Vec::with_capacity(64);
    let mut best = 0;
    let mut seen = vec![0u64; 1 << 20];
    stack.push((State { minutes: 30, opened: 0, index: 0 }, 0u64));

    while let Some((state, pressure)) = stack.pop() {
        if seen_before(&mut seen, state.pack(), pressure) {
            continue;
        }

        if state.minutes <= 0 || state.opened == all_valves {
            if pressure > best {
                best = pressure;
            }
            continue;
        }

        let valve = graph.node(state.index).unwrap();
        let (state, new_pressure) = state.with_open(valve.flow_rate);
        for (next_index, min) in graph.edges(state.index).unwrap() {
            if state.has_opened(*next_index) {
                continue;
            }

            stack.push((state.with_visit(*next_index, *min), pressure + new_pressure.0))
        }
    }

    best
}

#[allow(dead_code)]
fn part2(graph: &Graph<Valve, i8>) -> u64 {
    let mut dijkstra: Dijkstra<State2, Pressure> = Dijkstra::new(false, false);
    let initial_state = State2 {
        minutes: (26, 26),
        opened: 0,
        index: (0, 0),
    };
    let all_valves = (1 << graph.len()) - 1;

    let (pressure, _) = dijkstra.run(initial_state, Pressure(0), |state: &State2| {
        // Time's up
        if (state.minutes.0 <= 0 && state.minutes.1 <= 0) || state.opened == all_valves {
            return DijkstraResult::Success;
        }

        let mut next_steps = SmallVec::new();
        let elf_valve = graph.node(state.index.0).unwrap();
        let elephant_valve = graph.node(state.index.1).unwrap();
        let (next_state, new_pressure) = state.with_open(elf_valve, elephant_valve);

        if state.minutes.0 > state.minutes.1 {
            for (elf_index, dist) in graph.edges(state.index.0).unwrap() {
                if next_state.has_opened(*elf_index) {
                    continue;
                }

                let next_state = next_state.with_elf_visit(*elf_index, *dist);
                next_steps.push((new_pressure, Pressure(0), next_state));
            }
        } else {
            for (elephant_index, dist) in graph.edges(state.index.1).unwrap() {
                if next_state.has_opened(*elephant_index) {
                    continue;
                }

                let next_state = next_state.with_elephant_visit(*elephant_index, *dist);
                next_steps.push((new_pressure, Pressure(0), next_state));
            }
        }

        DijkstraResult::Continue(next_steps)
    }).unwrap();

    pressure.0
}

fn part2_dfs(graph: &Graph<Valve, i8>) -> u64 {
    let all_valves = (1 << graph.len()) - 1;
    let mut stack = Vec::with_capacity(64);
    let mut best = 0;
    let mut seen = vec![0u64; 1 << 24];

    stack.push((State2 { minutes: (26, 26), opened: 1, index: (0, 0) }, 0u64));

    while let Some((state, pressure)) = stack.pop() {
        if seen_before(&mut seen, state.pack(), pressure) {
            continue;
        }

        if (state.minutes.0 <= 0 && state.minutes.1 <= 0) || state.opened == all_valves {
            if pressure > best {
                best = pressure;
            }

            continue;
        }

        let elf_valve = graph.node(state.index.0).unwrap();
        let elephant_valve = graph.node(state.index.1).unwrap();
        let (next_state, new_pressure) = state.with_open(elf_valve, elephant_valve);

        if state.minutes.0 < 15 || state.minutes.1 < 15 {
            let mut total_potential = pressure + new_pressure.0;
            for (index, elf_dist) in graph.edges(state.index.0).unwrap() {
                if next_state.has_opened(*index) {
                    continue;
                }

                let elephant_dist = graph.edge(state.index.1, *index)
                    .unwrap_or(&0);
                let best_minute = max(max(
                    state.minutes.0 - *elf_dist,
                    state.minutes.1 - *elephant_dist,
                ), 0);

                let next_valve = graph.node(*index).unwrap();
                total_potential += next_valve.flow_rate * (best_minute as u64)
            }

            if total_potential <= best {
                continue;
            }
        }

        if state.minutes.0 > state.minutes.1 {
            for (elf_index, dist) in graph.edges(state.index.0).unwrap() {
                if next_state.has_opened(*elf_index) {
                    continue;
                }

                let next_state = next_state.with_elf_visit(*elf_index, *dist);
                stack.push((next_state, pressure + new_pressure.0));
            }
        } else {
            for (elephant_index, dist) in graph.edges(state.index.1).unwrap() {
                if next_state.has_opened(*elephant_index) {
                    continue;
                }

                let next_state = next_state.with_elephant_visit(*elephant_index, *dist);
                stack.push((next_state, pressure + new_pressure.0));
            }
        }
    }

    best
}

fn seen_before(seen: &mut Vec<u64>, seen_index: usize, pressure: u64) -> bool {
    if seen[seen_index] != 0 && seen[seen_index] > pressure {
        return true;
    }
    seen[seen_index] = pressure;

    return false;
}

#[derive(Clone, Debug)]
struct Valve {
    name: [u8; 2],
    flow_rate: u64,
}

#[derive(Clone, Copy, Debug, Eq)]
struct State {
    minutes: i8,
    index: usize,
    opened: u64,
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        (self.index, self.opened) ==
            (other.index, other.opened)
    }
}

impl Hash for State {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.index, self.opened).hash(state)
    }
}

impl State {
    fn pack(&self) -> usize {
        self.opened as usize | (self.index << 16)
    }

    fn has_opened(&self, index: usize) -> bool {
        (self.opened & 1 << index) != 0
    }

    fn with_open(&self, flow_rate: u64) -> (Self, Pressure) {
        let mut s = *self;
        s.opened |= 1 << self.index;

        let c = Pressure(flow_rate * s.minutes as u64);
        (s, c)
    }

    fn with_visit(&self, index: usize, min: i8) -> Self {
        let mut s = self.clone();
        s.index = index;
        s.minutes -= min;
        s
    }
}

#[derive(Clone, Copy, Debug, Eq)]
struct State2 {
    minutes: (i8, i8),
    index: (usize, usize),
    opened: u64,
}

fn minmax<T: Ord>((a, b): (T, T)) -> (T, T) {
    if a < b { (a, b) } else { (b, a) }
}

impl PartialEq for State2 {
    fn eq(&self, other: &Self) -> bool {
        if self.opened == self.opened {
            let self_index_sorted = minmax(self.index);
            let other_index_sorted = minmax(other.index);

            self_index_sorted == other_index_sorted
        } else {
            false
        }
    }
}

impl Hash for State2 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let (si1, si2) = minmax(self.index);
        (si1, si2, self.opened).hash(state)
    }
}

impl State2 {
    fn has_opened(&self, index: usize) -> bool {
        (self.opened & 1 << index) != 0
    }

    fn pack(&self) -> usize {
        let (si1, si2) = minmax(self.index);
        self.opened as usize | (si1 << 16) | (si2 << 20)
    }

    fn with_open(&self, elf_valve: &Valve, elephant_valve: &Valve) -> (Self, Pressure) {
        let mut s = *self;

        let mut pressure = 0u64;
        let elf_mask = 1 << self.index.0;
        let elephant_mask = 1 << self.index.1;

        if elf_mask & self.opened == 0 && self.minutes.0 > 0 {
            s.opened |= elf_mask;
            pressure += elf_valve.flow_rate * s.minutes.0 as u64
        }
        if elephant_mask & self.opened == 0 && self.minutes.1 > 0 {
            s.opened |= elephant_mask;
            pressure += elephant_valve.flow_rate * s.minutes.1 as u64
        }

        (s, Pressure(pressure))
    }

    fn with_elf_visit(&self, index: usize, min: i8) -> Self {
        let mut s = self.clone();
        s.index.0 = index;
        s.minutes.0 -= min;
        s
    }

    fn with_elephant_visit(&self, index: usize, min: i8) -> Self {
        let mut s = self.clone();
        s.index.1 = index;
        s.minutes.1 -= min;
        s
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
struct Pressure(u64);

impl Add for Pressure {
    type Output = Pressure;

    fn add(self, rhs: Self) -> Self::Output {
        Pressure(self.0 + rhs.0)
    }
}

impl PartialOrd<Self> for Pressure {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.0.cmp(&self.0))
    }
}

impl Ord for Pressure {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        other.0.cmp(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = include_bytes!("test_fixtures/d16_p1_example.txt");

    #[test] #[ignore]
    fn parse_works_on_example() {
        let g = parse(P1_EXAMPLE);

        println!("{:?}", g.nodes().collect::<Vec<(usize, &Valve)>>());

        assert_eq!(g.len(), 7);
        assert_eq!(g.find_by(|v| &v.name == b"AA"), Some(0));
//        assert_eq!(g.find_by(|v| &v.name == b"DD"), Some(1));

        let aa = g.node(g.find_by(|v| &v.name == b"AA").unwrap()).unwrap();
        let bb = g.node(g.find_by(|v| &v.name == b"BB").unwrap()).unwrap();
        let cc = g.node(g.find_by(|v| &v.name == b"CC").unwrap()).unwrap();
        let dd = g.node(g.find_by(|v| &v.name == b"DD").unwrap()).unwrap();
        let ee = g.node(g.find_by(|v| &v.name == b"EE").unwrap()).unwrap();
        let hh = g.node(g.find_by(|v| &v.name == b"HH").unwrap()).unwrap();
        let jj = g.node(g.find_by(|v| &v.name == b"JJ").unwrap()).unwrap();

        let aa_index = g.find_by(|v| &v.name == b"AA").unwrap();
        let cc_index = g.find_by(|v| &v.name == b"CC").unwrap();
        let hh_index = g.find_by(|v| &v.name == b"HH").unwrap();
        assert_eq!(g.edge(aa_index, cc_index), Some(&2));
        assert_eq!(g.edge(aa_index, hh_index), Some(&4));
        assert_eq!(g.edge(cc_index, hh_index), Some(&4));

        assert_eq!(aa.flow_rate, 0);
        assert_eq!(bb.flow_rate, 13);
        assert_eq!(cc.flow_rate, 2);
        assert_eq!(dd.flow_rate, 20);
        assert_eq!(ee.flow_rate, 3);
        assert_eq!(hh.flow_rate, 22);
        assert_eq!(jj.flow_rate, 21);
    }

    #[test] #[ignore]
    fn p1_works_on_example() {
        assert_eq!(part1_dfs(&parse(P1_EXAMPLE)), 1651);
    }

    #[test] #[ignore]
    fn p2_works_on_example() {
        assert_eq!(part2_dfs(&parse(P1_EXAMPLE)), 1707);
    }
}