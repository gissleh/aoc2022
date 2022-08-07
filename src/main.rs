#![feature(array_windows)]

pub mod year2015;
pub mod year2019;

fn main() {
    year2015::register_days();
    year2019::register_days();
}

