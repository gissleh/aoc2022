use common::aoc::Day;
use common::parse3::{Parser, signed_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.run(1, "", 50, || part1(input.clone()));
    day.run(2, "", 5, || part2(input.clone()));
}

fn parse(data: &[u8]) -> Vec<Number> {
    let mut res = signed_int::<i64>().and_discard(b'\n')
        .iterate(data)
        .enumerate()
        .map(|(i, n)| Number::new(n, i))
        .collect::<Vec<_>>();

    res.last_mut().unwrap().next = 0;
    res.first_mut().unwrap().prev = res.len() - 1;

    res
}

fn part1(ring: Vec<Number>) -> i64 {
    puzzle(ring, 1, 1)
}

fn part2(ring: Vec<Number>) -> i64 {
    puzzle(ring, 811589153, 10)
}

fn puzzle(mut ring: Vec<Number>, key: i64, mixes: usize) -> i64 {
    if key != 1 {
        for v in ring.iter_mut() {
            v.value *= key;
        }
    }

    for _ in 0..mixes {
        #[cfg(test)] print_ring(&ring, "Start of mix");

        for i in 0..ring.len() {
            if ring[i].value == 0 {
                continue;
            }

            let Number { next, prev, .. } = ring[i];
            ring[prev].next = next;
            ring[next].prev = prev;

            if ring[i].value > 0 {
                let new_index = ring[i].move_right(ring[i].value as usize, true, &ring);

                // prev <-> new_index <-> next
                // new_index <-> i <-> next

                let next = ring[new_index].next;
                ring[new_index].next = i;
                ring[i].prev = new_index;
                ring[next].prev = i;
                ring[i].next = next;
            } else {
                let new_index = ring[i].move_left(-ring[i].value as usize, true, &ring);

                // prev <-> new_index <-> next
                // prev <-> i <-> new_index

                let prev = ring[new_index].prev;
                ring[new_index].prev = i;
                ring[i].next = new_index;
                ring[prev].next = i;
                ring[i].prev = prev;
            }

            #[cfg(test)] print_ring(&ring, "-");
        }
    }


    let mut current: usize = ring.iter().position(|v| v.value == 0).unwrap();
    let mut sum = 0;
    for _ in 0..3 {
        current = ring[current].move_right(1000, false, &ring);
        sum += ring[current].value;

        #[cfg(test)] println!("{}", ring[current].value);
    }

    sum
}

#[allow(dead_code)]
fn print_ring(ring: &[Number], msg: &'static str) {
    println!("{}:", msg);

    let mut index = 0;
    for i in 0..ring.len() {
        if i > 0 {
            print!(", ");
        }

        print!("{}({})", ring[index].value, index);

        index = ring[index].next;
    }
    println!();
    println!();
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Number {
    value: i64,
    prev: usize,
    next: usize,
}

impl Number {
    fn move_left<'a>(&self, n: usize, dislodged: bool, ring: &[Number]) -> usize {
        #[cfg(test)] assert!(n > 0);

        let d = dislodged as usize;
        let mut current = self.prev;
        for _ in 1-d..((n-d) % (ring.len() - d)) {
            current = ring[current].prev;
        }

        current
    }

    fn move_right<'a>(&self, n: usize, dislodged: bool, ring: &[Number]) -> usize {
        #[cfg(test)] assert!(n > 0);

        let d = dislodged as usize;
        let mut current = self.next;
        for _ in 1- d..((n- d) % (ring.len() - d)) {
            current = ring[current].next;
        }

        current
    }

    fn new(value: i64, index: usize) -> Self {
        let next = index + 1;
        let prev = index.wrapping_sub(1);

        Number {
            value,
            next,
            prev: if prev != usize::MAX { prev } else { 0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_RING: &[Number] = &[
        Number { value: 1, next: 1, prev: 6 },
        Number { value: 2, next: 2, prev: 0 },
        Number { value: -3, next: 3, prev: 1 },
        Number { value: 3, next: 4, prev: 2 },
        Number { value: -2, next: 5, prev: 3 },
        Number { value: 0, next: 6, prev: 4 },
        Number { value: 4, next: 0, prev: 5 },
    ];

    #[test]
    fn parse_works_on_example() {
        assert_eq!(
            parse(include_bytes!("./test_fixtures/d20_p1_example.txt")).as_slice(),
            EXAMPLE_RING
        );
    }

    #[test]
    fn ring_moves_right_index() {
        assert_eq!(EXAMPLE_RING[0].move_right(1, false, &EXAMPLE_RING), 1);
        assert_eq!(EXAMPLE_RING[0].move_right(3, false, &EXAMPLE_RING), 3);
        assert_eq!(EXAMPLE_RING[0].move_right(23, false, &EXAMPLE_RING), 2);
        assert_eq!(EXAMPLE_RING[0].move_left(1, false, &EXAMPLE_RING), 6);
        assert_eq!(EXAMPLE_RING[0].move_left(2, false, &EXAMPLE_RING), 5);
        assert_eq!(EXAMPLE_RING[0].move_left(17, false, &EXAMPLE_RING), 4);
    }

    #[test]
    fn p1_works_on_example() {
        assert_eq!(part1(EXAMPLE_RING.to_owned()), 3);
    }

    #[test]
    fn p2_works_on_example() {
        assert_eq!(part2(EXAMPLE_RING.to_owned()), 1623178306);
    }
}