mod day02;
mod day05;
mod day09;
mod day20;
mod day24;

pub fn register_days() {
    common::register_day!(2019, 2, day02);
    common::register_day!(2019, 5, day05);
    common::register_day!(2019, 9, day09);
    common::register_day!(2019, 20, day20);
    common::register_day!(2019, 24, day24);
}