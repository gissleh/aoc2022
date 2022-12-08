use common::aoc::AOC;

mod day15;
mod day17;

pub fn main(aoc: &AOC) {
    aoc.run_day(15, day15::main);
    aoc.run_day(17, day17::main);
}