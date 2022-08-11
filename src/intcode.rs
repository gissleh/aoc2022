use crate::parse;

#[derive(Clone)]
pub struct Intcode<M> where M: Memory {
    memory: M,
    pos: usize,
    rb: i64,
}

impl<M> Intcode<M> where M: Memory {
    pub fn reset(&mut self, original: &M) {
        self.pos = 0;
        self.rb = 0;
        self.memory.reset(&original);
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    pub fn run_input(&mut self, inputs: &[i64]) -> (IntcodeResult, usize) {
        let mut next: Option<i64> = None;
        let mut i = 0usize;

        loop {
            match self.run_step(next) {
                IntcodeResult::Continue => {}
                IntcodeResult::InputNeeded => {
                    if i < inputs.len() {
                        next = Some(inputs[i]);
                        i += 1;
                    } else {
                        return (IntcodeResult::InputNeeded, i);
                    }
                }
                other => { return (other, i); }
            }
        }
    }

    pub fn run(&mut self) -> IntcodeResult {
        loop {
            match self.run_step(None) {
                IntcodeResult::Continue => {}
                other => { return other; }
            }
        }
    }

    pub fn run_step(&mut self, input: Option<i64>) -> IntcodeResult {
        let opcode = self.memory.get(self.pos);
        let m1 = (opcode / 100) % 10;
        let m2 = (opcode / 1000) % 10;
        let m3 = (opcode / 10000) % 10;

        match opcode % 100 {
            1 => {
                let (a, b, dst) = self.memory.abd(self.pos, m1, m2, m3, self.rb);

                self.memory.set(dst, a + b);
                #[cfg(test)] println!("{} ADD({}, {}, >{})", self.pos, a, b, dst);
                self.pos += 4;

                IntcodeResult::Continue
            }
            2 => {
                let (a, b, dst) = self.memory.abd(self.pos, m1, m2, m3, self.rb);

                self.memory.set(dst, a * b);
                #[cfg(test)] println!("{} MUL({}, {}, >{})", self.pos, a, b, dst);
                self.pos += 4;

                IntcodeResult::Continue
            }
            3 => {
                if let Some(v) = input {
                    let dst = self.memory.addr(self.pos + 1, m1, self.rb);
                    self.memory.set(dst, v);
                    #[cfg(test)] println!("{} IN({}, >{}) {}", self.pos, v, dst, opcode);
                    self.pos += 2;

                    IntcodeResult::Continue
                } else {
                    IntcodeResult::InputNeeded
                }
            }
            4 => {
                let v = self.memory.param(self.pos + 1, m1, self.rb);
                #[cfg(test)] println!("{} OUT({}) m1={} rb={}", self.pos, v, m1, self.rb);
                self.pos += 2;

                IntcodeResult::Output(v)
            }
            5 => {
                #[cfg(test)] println!("{} JT({}, {})", self.pos, self.memory.param(self.pos + 1, m1, self.rb), self.memory.param(self.pos + 2, m1, self.rb));

                let v = self.memory.param(self.pos + 1, m1, self.rb);
                if v != 0 {
                    self.pos = self.memory.param(self.pos + 2, m2, self.rb) as usize
                } else {
                    self.pos += 3;
                }

                IntcodeResult::Continue
            }
            6 => {
                #[cfg(test)] println!("{} JF({}, {})", self.pos, self.memory.param(self.pos + 1, m1, self.rb), self.memory.param(self.pos + 2, m1, self.rb));

                let v = self.memory.param(self.pos + 1, m1, self.rb);
                if v == 0 {
                    self.pos = self.memory.param(self.pos + 2, m2, self.rb) as usize
                } else {
                    self.pos += 3;
                }

                IntcodeResult::Continue
            }
            7 => {
                let (a, b, dst) = self.memory.abd(self.pos, m1, m2, m3, self.rb);
                self.memory.set(dst, (a < b) as i64);
                #[cfg(test)] println!("{} LT({}, {}, >{})", self.pos, a, b, dst);
                self.pos += 4;

                IntcodeResult::Continue
            }
            8 => {
                let (a, b, dst) = self.memory.abd(self.pos, m1, m2, m3, self.rb);
                self.memory.set(dst, (a == b) as i64);
                #[cfg(test)] println!("{} EQ({}, {}, >{})", self.pos, a, b, dst);
                self.pos += 4;

                IntcodeResult::Continue
            }
            9 => {
                #[cfg(test)] println!("{} RB({}) was {}", self.pos, self.memory.param(self.pos + 1, m1, self.rb), self.rb);

                self.rb += self.memory.param(self.pos + 1, m1, self.rb);
                self.pos += 2;

                IntcodeResult::Continue
            }
            99 => IntcodeResult::Exit,
            invalid_opcode => IntcodeResult::InvalidOp(invalid_opcode),
        }
    }

    pub fn new(initial_program: &M) -> Self {
        Self {
            memory: initial_program.clone(),
            pos: 0,
            rb: 0,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum IntcodeResult {
    Continue,
    InputNeeded,
    Output(i64),
    InvalidOp(i64),
    Exit,
}

pub trait Memory: Clone {
    fn reset(&mut self, original: &Self);
    fn set(&mut self, i: usize, v: i64);
    fn get(&self, i: usize) -> i64;

    #[inline]
    fn addr(&self, i: usize, m: i64, rb: i64) -> usize {
        let v = match m {
            0 => self.get(i),
            2 => self.get(i) + rb,
            _ => unreachable!()
        };

        v as usize
    }

    #[inline]
    fn param(&self, i: usize, m: i64, rb: i64) -> i64 {
        match m {
            0 => self.get(self.get(i) as usize),
            1 => self.get(i),
            2 => self.get((self.get(i) + rb) as usize),
            _ => unreachable!()
        }
    }

    #[inline]
    fn abd(&self, i: usize, m1: i64, m2: i64, m3: i64, rb: i64) -> (i64, i64, usize) {
        let a = self.param(i + 1, m1, rb);
        let b = self.param(i + 2, m2, rb);
        let dst = self.addr(i + 3, m3, rb);

        return (a, b, dst);
    }
}

#[derive(Clone)]
pub struct FixedMemory<const S: usize> {
    data: [i64; S],
}

impl<const S: usize> FixedMemory<S> {
    pub fn from_arr(data: [i64; S]) -> Self {
        Self { data }
    }

    pub fn parse(mut input: &[u8]) -> Self {
        let mut index = 0usize;
        let mut res = [0i64; S];
        while let Some((i, _, next)) = crate::parse_all!( input, parse::int::<i64>, parse::byte ) {
            input = next;
            res[index] = i;
            index += 1;
        }

        Self::from_arr(res)
    }
}

impl<const S: usize> Memory for FixedMemory<S> {
    fn reset(&mut self, original: &Self) {
        self.data = original.data;
    }

    fn set(&mut self, i: usize, v: i64) {
        self.data[i] = v;
    }

    fn get(&self, i: usize) -> i64 {
        self.data[i]
    }
}

#[derive(Clone)]
pub struct FixedAndSparseMemory<const S: usize> {
    data: [i64; S],
    extras: Vec<(usize, i64)>,
}

impl<const S: usize> Memory for FixedAndSparseMemory<S> {
    fn reset(&mut self, original: &Self) {
        self.extras.clear();
        self.data = original.data;

        if !original.extras.is_empty() {
            self.extras.extend_from_slice(&original.extras);
        }
    }

    fn set(&mut self, i: usize, v: i64) {
        if i < S {
            self.data[i] = v;
        } else {
            if let Some(found_index) = self.extras.iter().position(|(i2, _)| *i2 == i) {
                self.extras[found_index].1 = v;
            } else {
                self.extras.push((i, v));
            }
        }
    }

    fn get(&self, i: usize) -> i64 {
        if i < S {
            return self.data[i];
        } else {
            if let Some((_, v)) = self.extras.iter().find(|(i2, _)| *i2 == i) {
                *v
            } else {
                0
            }
        }
    }
}

impl<const S: usize> FixedAndSparseMemory<S> {
    pub fn from_arr(data: [i64; S]) -> Self {
        Self { data, extras: Vec::new() }
    }

    pub fn parse(input: &[u8]) -> Self {
        let fm = FixedMemory::parse(input);

        Self { data: fm.data, extras: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;
    use super::*;

    fn run_io<const S: usize>(prog: [i64; S], inputs: &[i64]) -> IntcodeResult {
        let mut ic = Intcode::new(&FixedAndSparseMemory::from_arr(prog));
        let (res, _) = ic.run_input(inputs);

        res
    }

    fn run_io_mout<const S: usize>(prog: [i64; S], mut inputs: &[i64]) -> Vec<i64> {
        let mut ic = Intcode::new(&FixedAndSparseMemory::from_arr(prog));
        let mut res = Vec::with_capacity(8);

        loop {
            let (r, n) = ic.run_input(inputs);
            inputs = &inputs[n..];

            match r {
                IntcodeResult::Exit => { break; }
                IntcodeResult::Output(o) => { res.push(o); }
                _ => panic!("Unexpected op: {:?}", r),
            }
        }

        for (e, v) in ic.memory.extras {
            println!("Extra [{}] = {}", e, v);
        }

        res
    }

    #[test]
    fn d5_output() {
        let mut ic = Intcode::new(&FixedMemory::from_arr([4, 50, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1337]));
        assert_eq!(ic.run(), IntcodeResult::Output(1337));
    }

    #[test]
    fn d5_output_its_input() {
        let mut ic = Intcode::new(&FixedMemory::from_arr([3, 0, 4, 0, 99]));
        assert_eq!(ic.run(), IntcodeResult::InputNeeded);
        assert_eq!(ic.run(), IntcodeResult::InputNeeded);
        assert_eq!(ic.run_input(&[32, 64, 96]), (IntcodeResult::Output(32), 1));
        assert_eq!(ic.run(), IntcodeResult::Exit);
    }

    #[test]
    fn d5_immediate_mode() {
        let mut ic = Intcode::new(&FixedMemory::from_arr([1002, 4, 3, 4, 33]));
        assert_eq!(ic.run(), IntcodeResult::Exit);
        assert_eq!(ic.memory().get(4), 99);
    }

    #[test]
    fn d5_conditionals() {
        assert_eq!(run_io([3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[4]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8], &[8]), IntcodeResult::Output(1));
        assert_eq!(run_io([3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[8]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8], &[7]), IntcodeResult::Output(1));
        assert_eq!(run_io([3, 3, 1108, -1, 8, 3, 4, 3, 99], &[432]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 3, 1108, -1, 8, 3, 4, 3, 99], &[8]), IntcodeResult::Output(1));
        assert_eq!(run_io([3, 3, 1107, -1, 8, 3, 4, 3, 99], &[42]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 3, 1107, -1, 8, 3, 4, 3, 99], &[7]), IntcodeResult::Output(1));
    }

    #[test]
    fn d5_jump_tests() {
        assert_eq!(run_io([3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9], &[0]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9], &[4432]), IntcodeResult::Output(1));
        assert_eq!(run_io([3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], &[0]), IntcodeResult::Output(0));
        assert_eq!(run_io([3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1], &[129]), IntcodeResult::Output(1));
    }

    const D5_LARGER_EXAMPLE: [i64; 47] = [
        3, 21, 1008, 21, 8, 20, 1005, 20, 22, 107, 8, 21, 20, 1006, 20, 31,
        1106, 0, 36, 98, 0, 0, 1002, 21, 125, 20, 4, 20, 1105, 1, 46, 104,
        999, 1105, 1, 46, 1101, 1000, 1, 20, 4, 20, 1105, 1, 46, 98, 99
    ];

    #[test]
    fn d5_larger_example() {
        assert_eq!(run_io(D5_LARGER_EXAMPLE, &[-4382]), IntcodeResult::Output(999));
        assert_eq!(run_io(D5_LARGER_EXAMPLE, &[8]), IntcodeResult::Output(1000));
        assert_eq!(run_io(D5_LARGER_EXAMPLE, &[4329]), IntcodeResult::Output(1001));
    }

    const D9_QUINE: [i64; 16] = [109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99];
    const D9_16DIGITS: [i64; 8] = [1102, 34915192, 34915192, 7, 4, 7, 99, 0];
    const D9_BIGBOI: [i64; 3] = [104,1125899906842624,99];

    #[test]
    fn d9_examples() {
        assert_eq!(run_io_mout(D9_QUINE, &[]), D9_QUINE.as_slice());
        assert_matches!(run_io(D9_16DIGITS, &[]), IntcodeResult::Output(v) if v > 999_999_999_999_999);
        assert_eq!(run_io(D9_BIGBOI, &[]), IntcodeResult::Output(1125899906842624));
    }
}