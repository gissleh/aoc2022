#![feature(array_windows)]
#![feature(option_result_contains)]
#![feature(array_chunks)]

use common::aoc::AOC;

#[cfg(feature = "2015")]
pub mod year2015;
#[cfg(feature = "2018")]
pub mod year2018;
#[cfg(feature = "2019")]
pub mod year2019;

pub mod year2022;

fn main() {
    #[cfg(feature = "2015")]
    year2015::register_days();
    #[cfg(feature = "2018")]
    year2018::register_days();
    #[cfg(feature = "2019")]
    year2019::register_days();

    let aoc = AOC::new();

    aoc.run_year(2022, year2022::main);
}

