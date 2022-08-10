use std::marker::PhantomData;
use std::ops::Add;
use num::Signed;
use crate::parse;

#[derive(Clone)]
pub struct Intcode<M> where M: Memory {
    memory: M,
    pos: usize,
}

impl<M> Intcode<M> where M: Memory {
    pub fn reset(&mut self, original: &M) {
        self.pos = 0;
        self.memory.reset(&original);
    }

    pub fn memory(&self) -> &M {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut M {
        &mut self.memory
    }

    pub fn run(&mut self) -> IntcodeResult {
        loop {
            match self.run_step() {
                IntcodeResult::Continue => {}
                other => { return other; }
            }
        }
    }

    pub fn run_step(&mut self) -> IntcodeResult {
        match self.memory.get(self.pos) {
            1 => {
                let (a, b, dst) = self.memory.abd(self.pos);

                self.memory.set(dst, a + b);
                self.pos += 4;

                IntcodeResult::Continue
            }
            2 => {
                let (a, b, dst) = self.memory.abd(self.pos);

                self.memory.set(dst, a * b);
                self.pos += 4;

                IntcodeResult::Continue
            }
            99 => IntcodeResult::Exit,
            _ => IntcodeResult::Error,
        }
    }

    pub fn new(initial_program: &M) -> Self {
        Self {
            memory: initial_program.clone(),
            pos: 0,
        }
    }
}

pub enum IntcodeResult {
    Continue,
    InputNeeded,
    Error,
    Exit,
}

pub trait Memory: Clone {
    fn reset(&mut self, original: &Self);
    fn set(&mut self, i: usize, v: i64);
    fn get(&self, i: usize) -> i64;
    fn slice(&self, i: usize) -> &[i64];
    fn slice_mut(&mut self, i: usize) -> &mut [i64];

    #[inline]
    fn abd(&self, i: usize) -> (i64, i64, usize) {
        let a = self.get(i + 1) as usize;
        let b = self.get(i + 2) as usize;
        let dst = self.get(i + 3) as usize;
        let a = self.get(a);
        let b = self.get(b);

        return (a, b, dst)
    }
}

#[derive(Clone)]
pub struct FixedMemory<const S: usize> {
    data: [i64; S],
}

impl<const S: usize> FixedMemory<S> {
    pub fn from_arr(data: [i64; S]) -> Self {
        Self{ data }
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

    fn slice(&self, i: usize) -> &[i64] {
        &self.data[i..]
    }

    fn slice_mut(&mut self, i: usize) -> &mut [i64] {
        &mut self.data[i..]
    }
}