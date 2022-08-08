use common::graph::Graph;
use common::parse;
use common::search::{Dijkstra};
use common::search::DijkstraResult::{Continue, Success};

common::day!(parse, part1, part2, 10000, 100, 100);

pub fn part1(graph: &Graph<&[u8], u32>) -> u32 {
    let mut min = u32::MAX;
    let goal_mask = (1 << graph.len() as u32) - 1u32;

    let mut dijkstra = Dijkstra::new(true, true);

    for i in 0..graph.len() {
        let initial_mask = 1u32 << i as u32;

        let res = dijkstra.run((i, initial_mask), 0, |(pos, visit_mask)| {
            if *visit_mask == goal_mask {
                Success
            } else {
                Continue(graph.edges(*pos).unwrap().filter_map(|(new_pos, distance)| {
                    let city_mask = 1 << *new_pos as u32;

                    if visit_mask & city_mask == 0 {
                        Some((*distance, 0, (*new_pos, visit_mask | city_mask)))
                    } else {
                        None
                    }
                }).collect())
            }
        });

        if let Some((s, _)) = res {
            if s < min {
                min = s;
            }
        }
    }

    min
}

pub fn part2(graph: &Graph<&[u8], u32>) -> u32 {
    let mut min = 0i32;
    let goal_mask = (1 << graph.len() as u32) - 1u32;

    let mut dijkstra = Dijkstra::new(false, false);

    for i in 0..graph.len() {
        let initial_mask = 1u32 << i as u32;

        let res = dijkstra.run((i, initial_mask), 0i32, |(pos, visit_mask)| {
            if *visit_mask == goal_mask {
                Success
            } else {
                Continue(graph.edges(*pos).unwrap().filter_map(|(new_pos, distance)| {
                    let city_mask = 1 << *new_pos as u32;

                    if visit_mask & city_mask == 0 {
                        Some((-(*distance as i32), 0, (*new_pos, visit_mask | city_mask)))
                    } else {
                        None
                    }
                }).collect())
            }
        });

        if let Some((s, _)) = res {
            if s < min {
                min = s;
            }
        }
    }

    -min as u32
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