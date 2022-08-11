use common::intcode::{FixedMemory, Intcode, IntcodeResult, Memory};

common::day!(parse, part1, part2, 100, 10000, 100);

fn part1<M>(program: &M) -> i64 where M: Memory {
    puzzle(program, &[1])
}

fn part2<M>(program: &M) -> i64 where M: Memory {
    puzzle(program, &[2])
}

#[inline]
fn puzzle<M>(program: &M, inputs: &[i64]) -> i64 where M: Memory {
    let mut ic = Intcode::new(program);

    let (r, n) = ic.run_input(inputs);
    match r {
        IntcodeResult::Output(v) => {
            assert_eq!(n, 1);
            v
        },
        _ => panic!("Program did not output r={:?} n={}", r, n)
    }
}

fn parse(input: &[u8]) -> FixedMemory<1280> {
    FixedMemory::parse(input)
}