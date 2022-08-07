use common::search::{astar, AStarState};
use common::graph::Graph;
use common::parse;

common::day!(parse, part1, part2, 10000, 100, 100);

pub fn part1(graph: &Graph<&[u8], u32>) -> u32 {
    let mut min = u32::MAX;
    let goal_mask = (1 << graph.len() as u32) - 1u32;

    for i in 0..graph.len() {
        let initial_visit = 1u32 << i as u32;
        let tm = TravelingSalesman{visit_map: initial_visit, pos: i};

        if let Some((s, _)) = astar(graph, &goal_mask, tm, 0, true, false) {
            if s < min {
                min = s;
            }
        }
    }

    min
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
struct TravelingSalesman {
    visit_map: u32,
    pos: usize,
}

impl AStarState<Graph<&[u8], u32>, u32, u32> for TravelingSalesman {
    fn heuristic(&self, _graph: &Graph<&[u8], u32>, _goal: &u32) -> u32 {
        0
    }

    fn is_goal(&self, _graph: &Graph<&[u8], u32>, goal: &u32) -> bool {
        self.visit_map.eq(goal)
    }

    fn next(&self, graph: &Graph<&[u8], u32>, buffer: &mut Vec<(u32, Self)>) {
        for (new_pos, cost) in graph.edges(self.pos).unwrap() {
            let mask = 1 << *new_pos as u32;

            if self.visit_map & mask == 0 {
                buffer.push((*cost, TravelingSalesman{
                    pos: *new_pos,
                    visit_map: self.visit_map | mask,
                }));
            }
        }
    }
}

pub fn part2(graph: &Graph<&[u8], u32>) -> u32 {
    let mut min = 0;
    let goal_mask = (1 << graph.len() as u32) - 1u32;
    for i in 0..graph.len() {
        let initial_visit = 1u32 << i as u32;
        let tm = WastefulSalesman{visit_map: initial_visit, pos: i};

        if let Some((s, _)) = astar(graph, &goal_mask, tm, 0, false, false) {
            if s < min {
                min = s;
            }
        }
    }

    -min as u32
}

#[derive(Hash, Copy, Clone, Eq, PartialEq)]
struct WastefulSalesman {
    visit_map: u32,
    pos: usize,
}

impl AStarState<Graph<&[u8], u32>, u32, i32> for WastefulSalesman {
    fn heuristic(&self, _graph: &Graph<&[u8], u32>, _goal: &u32) -> i32 {
        0
    }

    fn is_goal(&self, _graph: &Graph<&[u8], u32>, goal: &u32) -> bool {
        self.visit_map.eq(goal)
    }

    fn next(&self, graph: &Graph<&[u8], u32>, buffer: &mut Vec<(i32, Self)>) {
        for (new_pos, cost) in graph.edges(self.pos).unwrap() {
            let mask = 1 << *new_pos as u32;

            if self.visit_map & mask == 0 {
                buffer.push((-(*cost as i32), WastefulSalesman{
                    pos: *new_pos,
                    visit_map: self.visit_map | mask,
                }));
            }
        }
    }
}

pub fn parse(mut input: &[u8]) -> Graph<&[u8], u32> {
    let mut graph = Graph::new();
    while let Some((city_a, _, city_b, _, dist, _, next)) = common::parse_all!(
        input,
        parse::word,
        parse::expect_bytes(b"to "),
        parse::word,
        parse::expect_bytes(b"= "),
        parse::uint::<u32>,
        parse::line
    ) {
        let a = graph.find_or_insert(city_a);
        let b = graph.find_or_insert(city_b);
        graph.connect_mutual(a, b, dist);

        input = next
    }

    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &'static [u8] = b"London to Dublin = 464
London to Belfast = 518
Dublin to Belfast = 141
";

    #[test]
    fn p1_example() {
        let graph = parse(P1_EXAMPLE);
        assert_eq!(graph.len(), 3);
        assert_eq!(graph.edge(0, 1).cloned(), Some(464));
        assert_eq!(graph.edge(1, 0).cloned(), Some(464));
        assert_eq!(graph.edge(0, 2).cloned(), Some(518));
        assert_eq!(graph.edge(2, 0).cloned(), Some(518));
        assert_eq!(graph.edge(1, 2).cloned(), Some(141));
        assert_eq!(graph.edge(2, 1).cloned(), Some(141));
        assert_eq!(part1(&graph), 605);
    }
}