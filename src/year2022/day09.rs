use rustc_hash::FxHashSet;
use common::aoc::{Day, ResultPair};
use common::geo::Point;
use common::parse3;
use common::parse3::{expect_byte, Parser, unsigned_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Input Lines", input.len());
    day.note("Input Distance", input.iter().map(|(_, v)| *v).sum::<u32>());

    day.run(3, "", 1000, || both_parts(&input));
}

fn parse(data: &[u8]) -> Vec<(Point<i32>, u32)> {
    parse3::any_byte()
        .and_discard(expect_byte(b' '))
        .and(unsigned_int::<u32>())
        .skip(expect_byte(b'\n'))
        .map(|(d, n)| {
            match d {
                b'U' => (Point(0, -1), n),
                b'D' => (Point(0, 1), n),
                b'L' => (Point(-1, 0), n),
                b'R' => (Point(1, 0), n),
                _ => unreachable!()
            }
        })
        .repeat()
        .parse(data).unwrap()
}

fn both_parts(input: &[(Point<i32>, u32)]) -> ResultPair<usize, usize> {
    let mut has_seen_p1 = FxHashSet::default();
    let mut has_seen_p2 = FxHashSet::default();
    let mut rope = [Point(0i32, 0); 10];
    has_seen_p1.insert(Point(0, 0));
    has_seen_p2.insert(Point(0, 0));

    for (dir, count) in input {
        for _ in 0..*count {
            rope[0] += *dir;

            for i in 1..10usize {
                if let Some(new_tail) = find_tail_move(&rope[i - 1], &rope[i]) {
                    rope[i] = new_tail;

                    if i == 9 {
                        has_seen_p2.insert(new_tail);
                    } else if i == 1 {
                        has_seen_p1.insert(new_tail);
                    }
                }
            }
        }
    }

    ResultPair(has_seen_p1.len(), has_seen_p2.len())
}

fn find_tail_move(head: &Point<i32>, tail: &Point<i32>) -> Option<Point<i32>> {
    let head_square = head.surrounding_rect_inclusive(1);
    if head_square.contains_point_inclusive(tail) {
        return None;
    }
    if head.0 == tail.0 {
        if head.1 > tail.1 {
            Some(Point(head.0, head.1 - 1))
        } else {
            Some(Point(head.0, head.1 + 1))
        }
    } else if head.1 == tail.1 {
        if head.0 > tail.0 {
            Some(Point(head.0 - 1, head.1))
        } else {
            Some(Point(head.0 + 1, head.1))
        }
    } else {
        tail.diagonals()
            .into_iter()
            .find(|v| head_square.contains_point_inclusive(v))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_tail_move_works_on_examples() {
        // Tail at head or next to it.
        assert_eq!(find_tail_move(&Point(1, 1), &Point(1, 1)), None);
        assert_eq!(find_tail_move(&Point(1, 2), &Point(1, 1)), None);
        assert_eq!(find_tail_move(&Point(2, 2), &Point(1, 1)), None);

        // Move row-wise or column-wise
        assert_eq!(find_tail_move(&Point(3, 1), &Point(1, 1)), Some(Point(2, 1)));
        assert_eq!(find_tail_move(&Point(1, -1), &Point(1, 1)), Some(Point(1, 0)));
        assert_eq!(find_tail_move(&Point(1, 3), &Point(1, 1)), Some(Point(1, 2)));
        assert_eq!(find_tail_move(&Point(-1, 1), &Point(1, 1)), Some(Point(0, 1)));

        // Move diagonally
        assert_eq!(find_tail_move(&Point(2, 3), &Point(1, 1)), Some(Point(2, 2)));
        assert_eq!(find_tail_move(&Point(2, -1), &Point(1, 1)), Some(Point(2, 0)));
    }
}