use std::cmp::{max, min};
use num::abs;
use smallvec::SmallVec;
use common::aoc::Day;
use common::geo::Point;
use common::parse3::{Parser, signed_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Sensors", input.len());
    day.note("P2 points", input.iter()
        .flat_map(|i| i.position.manhattan_diamond(i.range + 1))
        .count());

    day.run(1, "", 10000, || part1(&input, 2000000));
    day.run(1, "slow", 10, || part1_slow(&input, 2000000));
    day.run(2, "", 10, || part2(&input, 4000000));
}

fn parse(input: &[u8]) -> Vec<Sensor> {
    Sensor::parser()
        .skip(b'\n')
        .repeat()
        .parse(input).unwrap()
}

fn part1(sensors: &[Sensor], y: i32) -> i32 {
    let mut ranges: SmallVec<[(i32, i32); 64]> = SmallVec::new();
    for sensor in sensors.iter() {
        if let Some(range) = sensor.range_at(y) {
            let mut overlapped = false;
            for i in 0..ranges.len() {
                let (cut, skip) = cut_range(ranges[i], range);
                if let Some(cut) = cut {
                    ranges[i] = cut;
                }

                if skip {
                    overlapped = true;
                }
            }

            if !overlapped {
                ranges.push(range);
            }
        }
    }

    // Lazy bugfix: remove overlapping ranges.
    for i in 0..ranges.len() {
        for j in 0..i {
            let (v, eaten) = cut_range(ranges[j], ranges[i]);
            assert!(v.is_none());
            if eaten {
                ranges[i] = (i32::MAX, i32::MAX);
                break;
            }
        }
    }

    ranges.iter().map(|(a, b)| b - a).sum::<i32>() - count_at_y(sensors, y)
}

fn part1_slow(sensors: &[Sensor], y: i32) -> i32 {
    let (min_x, max_x) = sensors.iter()
        .fold((0, 0), |(min_x, max_x), s| (
            min(s.position.0 - s.range, min_x),
            max(s.position.0 + s.range + 1, max_x),
        ));

    let mut c = 0;
    for x in min_x..max_x {
        let p = Point(x, y);
        for s in sensors.iter() {
            if p.manhattan_distance(&s.position) <= s.range {
                c += 1;
                break;
            }
        }
    }

    c - count_at_y(sensors, y)
}

fn part2(sensors: &[Sensor], size: i32) -> i64 {
    let found_point = sensors.iter().enumerate()
        .filter(|(i, s)| sensors.iter().skip(*i + 1)
            .find(|s2| {
                s.position.manhattan_distance(&s2.position) == (s.range + s2.range + 2)
            }).is_some())
        .flat_map(|(_, s)| s.position.manhattan_diamond(s.range + 1))
        .filter(|p| p.0 >= 0 && p.0 <= size && p.1 >= 0 && p.1 <= size)
        .find(|p| sensors.iter()
            .find(|s| s.position.manhattan_distance(&p) <= s.range)
            .is_none())
        .unwrap();

    (found_point.0 as i64 * 4000000) + found_point.1 as i64
}

fn count_at_y(sensors: &[Sensor], y: i32) -> i32 {
    let sensors_at_y = sensors.iter()
        .enumerate()
        .filter(|(_, s)| s.position.1 == y)
        .filter(|(i, s)| sensors[..*i].iter()
            .find(|s2| s.position.0 == s2.position.0)
            .is_none())
        .count() as i32;

    let beacons_at_y = sensors.iter()
        .enumerate()
        .filter(|(_, s)| s.nearest_beacon.1 == y)
        .filter(|(i, s)| sensors[..*i].iter()
            .find(|s2| s.nearest_beacon.0 == s2.nearest_beacon.0)
            .is_none())
        .count() as i32;

    sensors_at_y + beacons_at_y
}

fn cut_range(existing: (i32, i32), curr: (i32, i32)) -> (Option<(i32, i32)>, bool) {
    let (e1, e2) = existing;
    let (c1, c2) = curr;

    if c2 <= e1 || c1 >= e2 { // Entirely outside
        (None, false)
    } else if c1 >= e1 && c2 <= e2 { // Entirely inside
        (None, true)
    } else if e1 >= c1 && e2 <= c2 { // Entirely overlapping
        (Some((c1, c2)), true)
    } else if c2 < e2 { // Overlaps left
        (Some((c2, e2)), false)
    } else if c1 < e2 { // Overlaps right
        (Some((e1, c1)), false)
    } else {
        panic!("Edge case")
    }
}

struct Sensor {
    position: Point<i32>,
    nearest_beacon: Point<i32>,
    range: i32,
}

impl Sensor {
    fn range_at(&self, y: i32) -> Option<(i32, i32)> {
        let dist = abs(y - self.position.1);
        if dist <= self.range {
            let range_x = self.range - dist;
            Some((self.position.0 - range_x, self.position.0 + range_x + 1))
        } else {
            None
        }
    }

    fn parser<'i>() -> impl Parser<'i, Sensor> {
        // Sensor at x=3540455, y=2469135: closest beacon is at x=3866712, y=2438950
        b"Sensor at x="
            .and_instead(signed_int::<i32>())
            .and_discard(b", y=")
            .and(signed_int::<i32>())
            .and_discard(b": closest beacon is at x=")
            .and(signed_int::<i32>())
            .and_discard(b", y=")
            .and(signed_int::<i32>())
            .map(|(((sx, sy), bx), by)| {
                let position = Point(sx, sy);
                let nearest_beacon = Point(bx, by);

                Sensor {
                    range: position.manhattan_distance(&nearest_beacon),
                    position,
                    nearest_beacon,
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use common::grid2::VecGrid;
    use super::*;

    const P1_EXAMPLE: &[u8] = include_bytes!("./test_fixtures/d13_p1_example.txt");

    #[test]
    fn range_at() {
        let sensor = Sensor::parser().parse(b"Sensor at x=8, y=7: closest beacon is at x=2, y=10").unwrap();

        assert_eq!(sensor.range_at(-3), None);
        assert_eq!(sensor.range_at(-2), Some((8, 9)));
        assert_eq!(sensor.range_at(0), Some((6, 11)));
        assert_eq!(sensor.range_at(7), Some((-1, 18)));
        assert_eq!(sensor.range_at(9), Some((1, 16)));
        assert_eq!(sensor.range_at(10), Some((2, 15)));
        assert_eq!(sensor.range_at(11), Some((3, 14)));
        assert_eq!(sensor.range_at(13), Some((5, 12)));
        assert_eq!(sensor.range_at(15), Some((7, 10)));
        assert_eq!(sensor.range_at(16), Some((8, 9)));
        assert_eq!(sensor.range_at(17), None);
    }

    #[test]
    fn p1_ranges() {
        let sensors = parse(P1_EXAMPLE);
        let mut ranges = Vec::new();

        for sensor in sensors.iter() {
            let mut overlapped = false;
            if let Some(range) = sensor.range_at(10) {
                for i in 0..ranges.len() {
                    let (cut, skip) = cut_range(ranges[i], range);
                    if let Some(a) = cut {
                        ranges[i] = a;
                    }

                    if skip {
                        overlapped = true;
                    }
                }

                if !overlapped {
                    ranges.push(range);
                }
            }
        }

        let mut viz = [' '; 64];
        for (r1, r2) in ranges.iter() {
            for x in *r1..*r2 {
                assert_eq!(viz[(x + 16) as usize], ' ');
                viz[(x + 16) as usize] = '#';
            }
        }
        println!("{}", viz.iter().collect::<String>());

        let mut viz_e = [' '; 64];
        for x in -2..25i32 {
            viz_e[(x + 16) as usize] = '^';
        }
        println!("{}", viz_e.iter().collect::<String>());

        assert_eq!(ranges.iter().map(|(a, b)| b - a).sum::<i32>(), b"####B######################".len() as i32);

        for i in 0..ranges.len() {
            for j in 0..i {
                assert_eq!(cut_range(ranges[j], ranges[i]), (None, false));
            }
        }
    }

    #[test]
    fn part1_example() {
        assert_eq!(part1(&parse(P1_EXAMPLE), 9), 25);
        assert_eq!(part1(&parse(P1_EXAMPLE), 10), 26);
        assert_eq!(part1(&parse(P1_EXAMPLE), 11), 27);
    }

    #[test]
    fn part1_slow_example() {
        assert_eq!(part1_slow(&parse(P1_EXAMPLE), 9), 25);
        assert_eq!(part1_slow(&parse(P1_EXAMPLE), 10), 26);
        assert_eq!(part1_slow(&parse(P1_EXAMPLE), 11), 27);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(&parse(P1_EXAMPLE), 20), 56000011);
    }


    #[test]
    fn cut_range_cases() {
        // existing covers curr entirely
        assert_eq!(cut_range((10, 15), (12, 14)), (None, true));
        assert_eq!(cut_range((10, 15), (12, 15)), (None, true));
        assert_eq!(cut_range((10, 15), (10, 14)), (None, true));
        // Curr covers existing entirely
        assert_eq!(cut_range((10, 15), (5, 20)), (Some((5, 20)), true));
        assert_eq!(cut_range((10, 15), (10, 20)), (Some((10, 20)), true));
        assert_eq!(cut_range((10, 15), (5, 15)), (Some((5, 15)), true));
        // Curr eats into existing
        assert_eq!(cut_range((10, 15), (12, 16)), (Some((10, 12)), false));
        assert_eq!(cut_range((10, 15), (7, 12)), (Some((12, 15)), false));
        // Curr does not touch existing
        assert_eq!(cut_range((10, 15), (-159, 9)), (None, false));
        assert_eq!(cut_range((10, 15), (7, 10)), (None, false));
        assert_eq!(cut_range((10, 15), (15, 20)), (None, false));
        assert_eq!(cut_range((10, 15), (16, 432)), (None, false));
    }
}