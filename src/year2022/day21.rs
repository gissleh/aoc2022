use rustc_hash::FxHashMap;
use common::aoc::Day;
use common::parse3::{choice, n_bytes_array, Parser, signed_int};
use common::search::bisect;

const ROOT: &[u8; 4] = b"root";
const HUMN: &[u8; 4] = b"humn";

pub fn main(day: &mut Day, input: &[u8]) {
    let (monkeys, root_index, humn_index) = day.run_parse(1000, || parse(input));

    day.run(1, "", 10000, || part1(&monkeys, root_index));
    day.run(2, "", 10000, || part2(&monkeys, root_index, humn_index));
    day.run(2, "rev, wrong", 10000, || part2_rev(&monkeys, root_index, humn_index));

    day.select_label("");
}

fn parse(data: &[u8]) -> (Vec<Monkey>, usize, usize) {
    let mut monkeys = Vec::with_capacity(1024);
    let mut index_map = FxHashMap::default();
    for v in MonkeyDescription::parser().whole_line().iterate(data) {
        index_map.insert(v.name, monkeys.len());
        monkeys.push(v);
    }

    let res = monkeys.into_iter()
        .map(|v| match v.job {
            JobDescription::Number(v) => Monkey::Number(v),
            JobDescription::Result(op, a, b) => Monkey::Result(
                op, index_map[&a], index_map[&b],
            )
        })
        .collect::<Vec<_>>();

    (res, index_map[ROOT], index_map[HUMN])
}

fn part1(list: &[Monkey], root_index: usize) -> i64 {
    list[root_index].number(list)
}

fn part2(list: &[Monkey], root_index: usize, humn_index: usize) -> i64 {
    if let Monkey::Result(_, a, b) = list.get(root_index).unwrap() {
        let (human_monkey, fixed_monkey) = if list[*a].find_monkey(&list, humn_index) {
            (*a, *b)
        } else {
            (*b, *a)
        };

        let expected = list[fixed_monkey].number(list);
        let first = list[human_monkey].number_h(list, humn_index, 1000);
        let second = list[human_monkey].number_h(list, humn_index, 2000);

        let mut res = bisect(1125899906842624, 562949953421312, |v| {
            let ord = list[human_monkey].number_h(list, humn_index, v).cmp(&expected);

            if first > second { ord.reverse() } else { ord }
        }).unwrap();

        while list[human_monkey].number_h(list, humn_index, res - 1) == expected {
            res -= 1;
        }

        res
    } else {
        panic!("Invalid root monkey")
    }
}

fn part2_rev(list: &[Monkey], root_index: usize, humn_index: usize) -> i64 {
    if let Monkey::Result(_, a, b) = list.get(root_index).unwrap() {
        let (human_monkey, fixed_monkey) = if list[*a].find_monkey(&list, humn_index) {
            (*a, *b)
        } else {
            (*b, *a)
        };

        let expected = list[fixed_monkey].number(list);
        let res = list[human_monkey].number_rev(expected, humn_index, list);

        res
    } else {
        panic!("Invalid root monkey")
    }
}

#[derive(Clone, Eq, PartialEq, Debug)]
enum Monkey {
    Number(i64),
    Result(Op, usize, usize),
}

impl Monkey {
    fn find_monkey(&self, list: &[Monkey], needle: usize) -> bool {
        match self {
            Monkey::Result(_, a, b) => {
                if *a == needle || *b == needle {
                    true
                } else {
                    list[*a].find_monkey(list, needle)
                        || list[*b].find_monkey(list, needle)
                }
            }
            Monkey::Number(_) => false,
        }
    }

    fn number_rev(&self, v: i64, humn_index: usize, list: &[Monkey]) -> i64 {
        match self {
            Monkey::Result(op, a, b) => {
                let (h, f) = if list[*a].find_monkey(list, humn_index) {
                    (*a, *b)
                } else {
                    (*b, *a)
                };

                assert!(h == humn_index || list[h].find_monkey(list, humn_index));
                assert!(!list[f].find_monkey(list, humn_index));

                let h = list.get(h).unwrap();
                let f = list.get(f).unwrap();

                let v = match *op {
                    Op::Add => v - f.number(list),
                    Op::Sub => v + f.number(list),
                    Op::Mul => v / f.number(list),
                    Op::Div => v * f.number(list),
                };

                h.number_rev(v, humn_index, list)
            }
            _ => {
                assert_eq!(*self, list[humn_index]);
                v
            }
        }
    }

    fn number_h(&self, list: &[Monkey], humn_index: usize, humn_number: i64) -> i64 {
        match self {
            Monkey::Result(op, a, b) => {
                let a = if *a == humn_index { humn_number } else {
                    list[*a].number_h(list, humn_index, humn_number)
                };
                let b = if *b == humn_index { humn_number } else {
                    list[*b].number_h(list, humn_index, humn_number)
                };

                match *op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => a / b,
                }
            }
            Monkey::Number(v) => *v,
        }
    }

    fn number(&self, list: &[Monkey]) -> i64 {
        match self {
            Monkey::Result(op, a, b) => {
                let a = list.get(*a).unwrap();
                let b = list.get(*b).unwrap();

                match *op {
                    Op::Add => a.number(list) + b.number(list),
                    Op::Sub => a.number(list) - b.number(list),
                    Op::Mul => a.number(list) * b.number(list),
                    Op::Div => a.number(list) / b.number(list),
                }
            }
            Monkey::Number(v) => *v,
        }
    }
}

struct MonkeyDescription {
    name: [u8; 4],
    job: JobDescription,
}

impl MonkeyDescription {
    fn parser<'i>() -> impl Parser<'i, MonkeyDescription> {
        n_bytes_array::<4>()
            .and_discard(b": ")
            .and(JobDescription::parser())
            .map(|(name, job)| MonkeyDescription { name, job })
    }
}

enum JobDescription {
    Number(i64),
    Result(Op, [u8; 4], [u8; 4]),
}

impl JobDescription {
    fn parser<'i>() -> impl Parser<'i, JobDescription> {
        choice((
            signed_int::<i64>()
                .map(|v| JobDescription::Number(v)),
            n_bytes_array::<4>()
                .and_discard(b' ')
                .and(choice([
                    b'+'.map_to_value(Op::Add),
                    b'-'.map_to_value(Op::Sub),
                    b'*'.map_to_value(Op::Mul),
                    b'/'.map_to_value(Op::Div),
                ]))
                .and_discard(b' ')
                .and(n_bytes_array::<4>())
                .map(|((a, op), b)| JobDescription::Result(op, a, b)),
        ))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Op { Add, Sub, Mul, Div }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn p1_works_on_example() {
        let (monkeys, root_index, _) = parse(include_bytes!("./test_fixtures/d21_example.txt"));
        assert_eq!(part1(&monkeys, root_index), 152);
    }

    #[test]
    fn p2_works_on_example() {
        let (monkeys, root_index, humn_index) = parse(include_bytes!("./test_fixtures/d21_example.txt"));
        assert_eq!(part2(&monkeys, root_index, humn_index), 301);
    }
}