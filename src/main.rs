#![feature(array_windows)]
#![feature(option_result_contains)]

#[cfg(feature = "2015")]
pub mod year2015;
#[cfg(feature = "2019")]
pub mod year2019;

fn main() {
    #[cfg(feature = "2015")]
    year2015::register_days();
    #[cfg(feature = "2019")]
    year2019::register_days();
}

