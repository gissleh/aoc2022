use std::collections::VecDeque;
use num::integer::lcm;
use common::aoc::Day;
use common::parse3::{choice, Parser, unsigned_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Monkeys", input.len());
    day.note("Items", input.iter().map(|m| m.items.len()).sum::<usize>());

    day.run(1, "", 10000, || part1(&input));
    day.run(2, "", 100, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<Monkey> {
    let mut monkeys = Monkey::parser()
        .repeat_delimited(b'\n')
        .parse(data).unwrap();

    monkeys.sort_unstable_by_key(|m| m.id);
    monkeys
}

fn part1(input: &[Monkey]) -> u64 {
    let mut inspected = vec![0u64; input.len()];
    let mut monkeys = input.to_vec();

    for _ in 0..20 {
        for i in 0..monkeys.len() {
            while let Some(worry_level) = monkeys[i].items.pop_front() {
                inspected[i] += 1;

                let new_worry_level = monkeys[i].operation.apply(worry_level) / 3;
                let next = if new_worry_level % monkeys[i].test_divisible_by == 0 {
                    monkeys[i].next_true
                } else {
                    monkeys[i].next_false
                };

                monkeys[next].items.push_back(new_worry_level);
            }
        }
    }

    // Get the two highest values.
    inspected.sort_unstable();
    inspected[inspected.len() - 2] * inspected[inspected.len() - 1]
}

fn part2(input: &[Monkey]) -> u64 {
    let mut inspected = vec![0u64; input.len()];
    let mut monkeys = input.to_vec();

    let mut lowest_common_multiple = 1;
    for m in monkeys.iter() {
        lowest_common_multiple = lcm(lowest_common_multiple, m.test_divisible_by);
    }

    for _ in 0..10000 {
        for i in 0..monkeys.len() {
            while let Some(worry_level) = monkeys[i].items.pop_front() {
                inspected[i] += 1;

                let new_worry_level = monkeys[i].operation.apply(worry_level) % lowest_common_multiple;
                let next = if new_worry_level % monkeys[i].test_divisible_by == 0 {
                    monkeys[i].next_true
                } else {
                    monkeys[i].next_false
                };

                monkeys[next].items.push_back(new_worry_level);
            }
        }
    }

    // Get the two highest values.
    inspected.sort_unstable();
    inspected[inspected.len() - 2] * inspected[inspected.len() - 1]
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Monkey {
    id: usize,
    items: VecDeque<u64>,
    operation: Operation,
    test_divisible_by: u64,
    next_true: usize,
    next_false: usize,
}

impl Monkey {
    #[inline]
    fn parser<'i>() -> impl Parser<'i, Monkey> {
        b"Monkey ".and_instead(unsigned_int())
            .and(b":\n  Starting items: "
                .and_instead(unsigned_int().repeat_delimited(b", "))
                .and_discard(b'\n'))
            .and(b"  Operation: new = old "
                .and_instead(choice((
                    b"* old".map_to_value(Operation::Square),
                    b"+ ".and_instead(unsigned_int()).map(Operation::Add),
                    b"* ".and_instead(unsigned_int()).map(Operation::Mul),
                    b"+ old".map_to_value(Operation::Double),
                )))
                .and_discard(b'\n'))
            .and(b"  Test: divisible by "
                .and_instead(unsigned_int::<u64>())
                .and_discard(b'\n'))
            .and(b"    If true: throw to monkey "
                .and_instead(unsigned_int::<usize>())
                .and_discard(b'\n'))
            .and(b"    If false: throw to monkey "
                .and_instead(unsigned_int::<usize>())
                .and_discard(b'\n'))
            .map(|(((((id, items), operation), test_divisible_by), next_true), next_false)| {
                Monkey { items: items.into(), id, operation, test_divisible_by, next_true, next_false }
            })
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Operation {
    Double,
    Square,
    Add(u64),
    Mul(u64),
}

impl Operation {
    #[inline]
    fn apply(&self, v: u64) -> u64 {
        match self {
            Operation::Square => v * v,
            Operation::Double => v + v,
            Operation::Add(w) => v + *w,
            Operation::Mul(w) => v * *w,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    const P1_MONKEYS: &[u8] = b"Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1
";

    #[test]
    fn parse_one_monkey() {
        let monkey = Monkey::parser().parse(P1_MONKEYS).unwrap();

        assert_eq!(monkey, Monkey {
            id: 0,
            items: vec![79u64, 98].into(),
            operation: Operation::Mul(19),
            test_divisible_by: 23,
            next_true: 2,
            next_false: 3,
        });
    }

    #[test]
    fn parse_input() {
        let monkeys = parse(P1_MONKEYS);

        assert_eq!(monkeys, vec![
            Monkey {
                id: 0,
                items: vec![79u64, 98].into(),
                operation: Operation::Mul(19),
                test_divisible_by: 23,
                next_true: 2,
                next_false: 3,
            },
            Monkey {
                id: 1,
                items: vec![54u64, 65, 75, 74].into(),
                operation: Operation::Add(6),
                test_divisible_by: 19,
                next_true: 2,
                next_false: 0,
            },
            Monkey {
                id: 2,
                items: vec![79u64, 60, 97].into(),
                operation: Operation::Square,
                test_divisible_by: 13,
                next_true: 1,
                next_false: 3,
            },
            Monkey {
                id: 3,
                items: vec![74u64].into(),
                operation: Operation::Add(3),
                test_divisible_by: 17,
                next_true: 0,
                next_false: 1,
            },
        ]);
    }

    #[test]
    fn part1_example() {
        assert_eq!(part1(&parse(P1_MONKEYS)), 10605);
    }

    #[test]
    fn part2_example() {
        assert_eq!(part2(&parse(P1_MONKEYS)), 2713310158);
    }
}