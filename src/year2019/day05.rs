use common::intcode::{FixedMemory, Intcode, IntcodeResult, Memory};

common::day!(parse, part1, part2, 100, 10000, 10000);

fn part1<M>(program: &M) -> i64 where M: Memory {
    puzzle(program, &[1])
}

fn part2<M>(program: &M) -> i64 where M: Memory {
    puzzle(program, &[5])
}

#[inline]
fn puzzle<M>(program: &M, mut inputs: &[i64]) -> i64 where M: Memory {
    let mut ic = Intcode::new(program);
    let mut last_output = 0;

    loop {
        let (res, n) = ic.run_input(&inputs);
        inputs = &inputs[n..];

        match res {
            IntcodeResult::Continue => {}
            IntcodeResult::Output(v) => { last_output = v }
            IntcodeResult::Exit => { break; }
            _ => { panic!("Unexpected result {:?}", res) }
        }
    }

    last_output
}

fn parse(input: &[u8]) -> FixedMemory<700> {
    FixedMemory::parse(input)
}