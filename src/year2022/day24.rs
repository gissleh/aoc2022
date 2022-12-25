use common::aoc::{Day, ResultPair};
use common::geo::Point;
use common::grid2::{FixedGrid, IterableSliceGrid, VecGrid};
use common::search2::{dijkstra, Search, WithCostHeuristic};

pub fn main(day: &mut Day, input: &[u8]) {
    let (blizzards, valley_width, valley_height) = day.run_parse(1000, || parse(input));

    day.note("Blizzards", blizzards.len());
    day.note("Valley Width", valley_width);
    day.note("Valley Height", valley_height);

    day.run(3, "", 5, || part1(&blizzards, valley_width, valley_height));
    day.run(3, "Masks", 5, || part1_masks(&blizzards, valley_width, valley_height));
}

fn parse(data: &[u8]) -> (Vec<(u8, Point<i16>)>, i16, i16) {
    let grid = VecGrid::parse_lines(data, b'\n').unwrap();

    let mut blizzards = Vec::with_capacity(4096);
    for (p, v) in grid.cells() {
        let pos = Point(p.0 as i16 - 1, p.1 as i16 - 1);
        match *v {
            b'^' => blizzards.push((0, pos)),
            b'<' => blizzards.push((1, pos)),
            b'>' => blizzards.push((2, pos)),
            b'v' => blizzards.push((3, pos)),
            _ => {}
        }
    }

    (blizzards, (grid.width() - 2) as i16, (grid.height() - 2) as i16)
}

fn part1(blizzards: &[(u8, Point<i16>)], width: i16, height: i16) -> ResultPair<u32, u32> {
    let right = width - 1;
    let bottom = height - 1;
    let goal = Point(right, bottom + 1);
    let start = Point(0, -1);

    let mut curr_h = blizzards.iter().filter(|(d, _)| *d == 1 || *d == 2).copied().collect::<Vec<_>>();
    let mut curr_v = blizzards.iter().filter(|(d, _)| *d == 0 || *d == 3).copied().collect::<Vec<_>>();
    let mut h_blizzards = Vec::with_capacity(width as usize);
    for _ in 0..width {
        h_blizzards.push(curr_h.clone());
        blow_winds(&mut curr_h, right, bottom);
    }
    let mut v_blizzards = Vec::with_capacity(height as usize);
    for _ in 0..height {
        v_blizzards.push(curr_v.clone());
        blow_winds(&mut curr_v, right, bottom);
    }

    let first_trip = run_search(start, 0, goal, width, height, &h_blizzards, &v_blizzards);
    let second_trip = run_search(goal, first_trip, start, width, height, &h_blizzards, &v_blizzards);
    let third_trip = run_search(start, first_trip + second_trip, goal, width, height, &h_blizzards, &v_blizzards);

    ResultPair(first_trip, first_trip + second_trip + third_trip)
}

fn run_search(start: Point<i16>, current_len: u32, goal: Point<i16>, width: i16, height: i16, h_blizzards: &[Vec<(u8, Point<i16>)>], v_blizzards: &[Vec<(u8, Point<i16>)>]) -> u32 {
    let right = width - 1;
    let bottom = height - 1;
    let initial_state = State {
        wind_index: (
            (current_len % width as u32) as u8,
            (current_len % height as u32) as u8,
        ),
        position: start,
    };

    dijkstra(WithCostHeuristic(initial_state, 0, 0))
        .run(|search, WithCostHeuristic(state, minute, _)| {
            let h_index = state.wind_index.0 as usize;
            let v_index = state.wind_index.1 as usize;

            let next_state = State {
                wind_index: (
                    (h_index + 1) as u8 % width as u8,
                    (v_index + 1) as u8 % height as u8,
                ),
                position: state.position,
            };

            for p in state.position.self_and_cardinals_offset(1) {
                if p == goal {
                    return Some(*minute);
                }

                if p.0 < 0 || p.1 < 0 || p.0 > right || p.1 > bottom {
                    if p != start {
                        continue;
                    }
                }

                let found = v_blizzards[v_index].iter()
                    .chain(h_blizzards[h_index].iter())
                    .find(|(_, b)| *b == p)
                    .is_some();

                if !found {
                    let mut next_state = next_state.clone();
                    next_state.position = p;
                    search.add_step(WithCostHeuristic(
                        next_state,
                        *minute + 1,
                        p.manhattan_distance(&goal) as u32,
                    ));
                }
            }

            None::<u32>
        })
        .next()
        .unwrap()
}

fn part1_masks(blizzards: &[(u8, Point<i16>)], width: i16, height: i16) -> ResultPair<u32, u32> {
    let right = width - 1;
    let bottom = height - 1;
    let goal = Point(right, bottom + 1);
    let start = Point(0, -1);

    let mut left_masks = vec![0u128; height as usize];
    let mut right_masks = vec![0u128; height as usize];
    let mut up_masks = vec![0u32; width as usize];
    let mut down_masks = vec![0u32; width as usize];
    for (dir, pos) in blizzards.iter() {
        match dir {
            0 => { up_masks[pos.0 as usize] |= 1 << pos.1 }
            1 => { left_masks[pos.1 as usize] |= 1 << pos.0 }
            2 => { right_masks[pos.1 as usize] |= 1 << pos.0 }
            3 => { down_masks[pos.0 as usize] |= 1 << pos.1 }
            _ => unreachable!()
        };
    }

    let mut h_masks = vec![Vec::with_capacity(height as usize); width as usize];
    let upper = 1 << right;
    for i in 0..(width as usize) {
        for (left_mask, right_mask) in left_masks.iter().zip(right_masks.iter()) {
            h_masks[i].push(*left_mask | *right_mask);
        }

        for m in left_masks.iter_mut() {
            if *m & 1 == 1 {
                *m >>= 1;
                *m |= upper;
            } else {
                *m >>= 1;
            }
        }
        for m in right_masks.iter_mut() {
            if *m & upper == upper {
                *m <<= 1;
                *m |= 1;
            } else {
                *m <<= 1;
            }
        }
    }

    let mut v_masks = vec![Vec::with_capacity(width as usize); height as usize];
    let upper = 1 << bottom;
    for i in 0..(height as usize) {
        for (up_mask, down_mask) in up_masks.iter().zip(down_masks.iter()) {
            v_masks[i].push(*up_mask | *down_mask);
        }

        for m in up_masks.iter_mut() {
            if *m & 1 == 1 {
                *m >>= 1;
                *m |= upper;
            } else {
                *m >>= 1;
            }
        }
        for m in down_masks.iter_mut() {
            if *m & upper == upper {
                *m <<= 1;
                *m |= 1;
            } else {
                *m <<= 1;
            }
        }
    }

    let first_trip = run_search_masks(start, 0, goal, width, height, &h_masks, &v_masks);
    let second_trip = run_search_masks(goal, first_trip, start, width, height, &h_masks, &v_masks);
    let third_trip = run_search_masks(start, first_trip + second_trip, goal, width, height, &h_masks, &v_masks);

    ResultPair(first_trip, first_trip + second_trip + third_trip)
}

fn run_search_masks(start: Point<i16>, current_len: u32, goal: Point<i16>, width: i16, height: i16, h_masks: &[Vec<u128>], v_masks: &[Vec<u32>]) -> u32 {
    let right = width - 1;
    let bottom = height - 1;
    let initial_state = State {
        wind_index: (
            (current_len % width as u32) as u8,
            (current_len % height as u32) as u8,
        ),
        position: start,
    };

    dijkstra(WithCostHeuristic(initial_state, 0, 0))
        .run(|search, WithCostHeuristic(state, minute, _)| {
            let h_index = state.wind_index.0 as usize;
            let v_index = state.wind_index.1 as usize;

            let next_state = State {
                wind_index: (
                    (h_index + 1) as u8 % width as u8,
                    (v_index + 1) as u8 % height as u8,
                ),
                position: state.position,
            };

            for p in state.position.self_and_cardinals_offset(1) {
                if p == goal {
                    return Some(*minute);
                }

                let occupied = if p == start {
                    false
                } else if p.0 < 0 || p.1 < 0 || p.0 > right || p.1 > bottom {
                    true
                } else {
                    let h_mask = 1u128 << p.0;
                    let v_mask = 1u32 << p.1;

                    h_masks[h_index][p.1 as usize] & h_mask != 0
                        || v_masks[v_index][p.0 as usize] & v_mask != 0
                };

                if !occupied {
                    let mut next_state = next_state.clone();
                    next_state.position = p;
                    search.add_step(WithCostHeuristic(
                        next_state,
                        *minute + 1,
                        p.manhattan_distance(&goal) as u32,
                    ));
                }
            }

            None::<u32>
        })
        .next()
        .unwrap()
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct State {
    position: Point<i16>,
    wind_index: (u8, u8),
}

fn blow_winds(blizzards: &mut Vec<(u8, Point<i16>)>, right: i16, bottom: i16) {
    for (dir, pos) in blizzards.iter_mut() {
        match dir {
            0 => if pos.1 == 0 { pos.1 = bottom } else { pos.1 -= 1; }
            1 => if pos.0 == 0 { pos.0 = right } else { pos.0 -= 1; }
            2 => if pos.0 == right { pos.0 = 0 } else { pos.0 += 1; }
            3 => if pos.1 == bottom { pos.1 = 0 } else { pos.1 += 1; }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = include_bytes!("./test_fixtures/d24_p1_example.txt");

    #[test]
    fn both_parts_works_on_example() {
        let (winds, w, h) = parse(P1_EXAMPLE);
        assert_eq!(part1(&winds, w, h), ResultPair(18, 54));
    }

    #[test]
    fn both_parts_masks_works_on_example() {
        let (winds, w, h) = parse(P1_EXAMPLE);
        assert_eq!(part1_masks(&winds, w, h), ResultPair(18, 54));
    }
}