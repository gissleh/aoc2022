use common::intcode::{FixedMemory, Intcode, Memory};

common::day!(parse, part1, part2, 100, 1000, 100);

fn part1(program: &FixedMemory<128>) -> i64 {
    let mut ic = Intcode::new(program);
    ic.memory_mut().set(1, 12);
    ic.memory_mut().set(2, 2);
    ic.run();
    ic.memory().get(0)
}

fn part2(program: &FixedMemory<128>) -> i64 {
    let mut ic = Intcode::new(program);

    for a in 0..99 {
        for b in 0..99 {
            ic.memory_mut().set(1, a);
            ic.memory_mut().set(2, b);
            ic.run();

            if ic.memory().get(0) == 19690720 {
                return a * 100 + b;
            }

            ic.reset(program);
        }
    }

    0
}

fn parse(input: &[u8]) -> FixedMemory<128> {
    FixedMemory::parse(input)
}