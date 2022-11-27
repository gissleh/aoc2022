use std::cmp::Ordering;
use common::intcode::{FixedMemory, Intcode, Memory};
use common::search::bisect;

common::day!(parse, part1, part2, 100, 1000, 100);

fn part1(program: &FixedMemory<128>) -> i64 {
    let mut ic = Intcode::new(program);
    ic.memory_mut().set(1, 12);
    ic.memory_mut().set(2, 2);
    ic.run();
    ic.memory().get(0)
}

fn part2(program: &FixedMemory<128>) -> i64 {
    let res = bisect(5000, 5000, move |curr| {
        if curr < 0 {
            return Ordering::Less
        } else if curr > 9999 {
            return Ordering::Greater
        }

        let mut ic = Intcode::new(program);
        ic.memory_mut().set(1, curr / 100);
        ic.memory_mut().set(2, curr % 100);
        ic.run();

        ic.memory().get(0).cmp(&19690720)
    });

    match res {
        Some(v) => v,
        None => 0,
    }
}

fn parse(input: &[u8]) -> FixedMemory<128> {
    FixedMemory::parse(input)
}