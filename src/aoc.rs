use std::fs::File;
use std::io::Read;
use chrono::Datelike;
use time::PreciseTime;

pub fn load_input(year: i32, date: u32) -> Vec<u8> {
    let args: Vec<String> = std::env::args().collect();
    let name = args.get(4).cloned().or(Some(format!("input/{}/day{:0width$}.txt", year, date, width=2))).unwrap();

    let mut buf = Vec::with_capacity(2048);
    match File::open(name.clone()) {
        Ok(mut file) => {
            file.read_to_end(&mut buf)
                .expect("Could not read file");
        }
        Err(e) => {
            panic!("Could not load file {}: {}", name, e);
        }
    }

    buf
}

pub fn get_year_and_date() -> (i32, u32) {
    let args: Vec<String> = std::env::args().collect();

    let mut year = chrono::Local::now().year();
    if let Some(arg1) = args.get(1) {
        if let Ok(v) = arg1.parse::<i32>() {
            year = v;
        }
    }

    let date = args.get(2).map(|v| v.parse::<u32>().unwrap()).or(Some(
        chrono::Local::now().day()
    )).unwrap();

    (year, date)
}

pub fn run_once<T>(callback: impl Fn() -> T) -> (T, i64) {
    let start = PreciseTime::now();
    let result = callback();
    let end = PreciseTime::now();

    (result, start.to(end).num_nanoseconds().unwrap())
}

pub fn run_many<T>(times: usize, callback: impl Fn() -> T) -> (T, i64) {
    if times == 1 {
        return run_once(callback);
    }

    let start = PreciseTime::now();
    let mut result= callback();
    for _ in 1..times {
        result = callback();
    }
    let end = PreciseTime::now();

    (
        result,
        start.to(end).num_nanoseconds().unwrap() / times as i64,
    )
}

pub fn format_duration(ns: i64) -> String {
    if ns > 10_000_000_000 {
        format!("{:.1}s", (ns as f64) / (1_000_000_000 as f64))
    } else if ns > 1_000_000_000 {
        format!("{:.2}s", (ns as f64) / (1_000_000_000 as f64))
    } else if ns > 1_000_000 {
        format!("{:.2}ms", (ns as f64) / (1_000_000 as f64))
    } else if ns > 1_000 {
        format!("{:.2}µs", (ns as f64) / (1_000 as f64))
    } else {
        format!("{}ns", ns)
    }
}


#[macro_export]
macro_rules! day {
    ($parse:ident, $part1:ident, $part2:ident, $parse_runs:expr, $part1_runs:expr, $part2_runs:expr) => {
        pub fn run_once_nobench(input: &[u8]) {
            let input = $parse(input);
            let part1_res = $part1(&input);
            let part2_res = $part2(&input);

            println!("Part 1:\n{}\n", part1_res);
            println!("Part 2:\n{}", part2_res);
        }

        pub fn run_once_bench(input: &[u8]) {
            let (input, input_ns) = common::aoc::run_once(|| $parse(input));
            let (part1_res, part1_ns) = common::aoc::run_once(|| $part1(&input));
            let (part2_res, part2_ns) = common::aoc::run_once(|| $part2(&input));

            println!("Part 1:\n{}\n", part1_res);
            println!("Part 2:\n{}\n", part2_res);
            println!("Parse: {}\nP1: {}\nP2: {}",
                common::aoc::format_duration(input_ns),
                common::aoc::format_duration(part1_ns),
                common::aoc::format_duration(part2_ns),
            );
        }

        pub fn run_many_bench(input: &[u8]) {
            let (input, input_ns) = common::aoc::run_many($parse_runs, || $parse(input));
            let (part1_res, part1_ns) = common::aoc::run_many($part1_runs, || $part1(&input));
            let (part2_res, part2_ns) = common::aoc::run_many($part1_runs, || $part2(&input));

            println!("Part 1:\n{}\n", part1_res);
            println!("Part 2:\n{}\n", part2_res);
            println!("Parse: {}\nP1: {}\nP2: {}",
                common::aoc::format_duration(input_ns),
                common::aoc::format_duration(part1_ns),
                common::aoc::format_duration(part2_ns),
            );
        }

        pub fn run_bench_table(label: String, input: &[u8]) {
            let (input, input_ns) = common::aoc::run_many($parse_runs, || $parse(input));
            let (_, part1_ns) = common::aoc::run_many($part1_runs, || $part1(&input));
            let (_, part2_ns) = common::aoc::run_many($part1_runs, || $part2(&input));

            println!("{:6} {: >10} {: >10} {: >10}",
                label,
                common::aoc::format_duration(input_ns),
                common::aoc::format_duration(part1_ns),
                common::aoc::format_duration(part2_ns),
            );
        }

        pub fn run_bench_table_once(label: String, input: &[u8]) {
            let (input, input_ns) = common::aoc::run_once(|| $parse(input));
            let (_, part1_ns) = common::aoc::run_once(|| $part1(&input));
            let (_, part2_ns) = common::aoc::run_once(|| $part2(&input));

            println!("{:6} {: >10} {: >10} {: >10}",
                label,
                common::aoc::format_duration(input_ns),
                common::aoc::format_duration(part1_ns),
                common::aoc::format_duration(part2_ns),
            );
        }
    }
}

#[macro_export]
macro_rules! register_day {
    ($year:expr, $date:expr, $module:ident) => {
        let (year, mut date) = common::aoc::get_year_and_date();

        if year == $year && (date == $date || date == 0) {
            if date == 0 {
                date = $date;
            }

            let args: Vec<String> = std::env::args().collect();
            let op = args.get(3).cloned().or(Some(String::from("run"))).unwrap();
            let buf = common::aoc::load_input(year, date);

            match op.as_str() {
                "run" => {
                    println!("--- Day {} ---------------", date);
                    $module::run_once_nobench(&buf);
                }
                "bench_once" => {
                    println!("--- Day {} ---------------", date);
                    $module::run_once_bench(&buf);
                }
                "bench" | "bench_many" => {
                    println!("--- Day {} ---------------", date);
                    $module::run_many_bench(&buf);
                }
                "table_once" => {
                    $module::run_bench_table_once(format!("Day {:0>2}", date), &buf);
                }
                "table" | "table_many" => {
                    $module::run_bench_table(format!("Day {:0>2}", date), &buf);
                }
                _ => {
                    panic!("Invalid op {}", op);
                }
            }
        }
    }
}
