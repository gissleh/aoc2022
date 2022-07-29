pub mod day01;
pub mod day02;
pub mod day03;
pub mod day04;
pub mod day05;
pub mod day06;
pub mod day07;

pub fn register_days() {
    common::register_day!(2015, 1, day01);
    common::register_day!(2015, 2, day02);
    common::register_day!(2015, 3, day03);
    common::register_day!(2015, 4, day04);
    common::register_day!(2015, 5, day05);
    common::register_day!(2015, 6, day06);
    common::register_day!(2015, 7, day07);
}