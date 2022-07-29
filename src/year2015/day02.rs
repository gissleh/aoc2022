use common::parse;
use std::cmp::min;

common::day!(parse, part1, part2, 10000, 10000, 10000);

fn parse(data: &[u8]) -> Vec<(u32, u32, u32)> {
    let mut boxes = Vec::with_capacity(64);
    let mut input = data;
    while let Some((w, _, l, _, h, _, new_input)) = common::parse_all!(
        input,
        parse::uint::<u32>, parse::expect_byte::<b'x'>,
        parse::uint::<u32>, parse::expect_byte::<b'x'>,
        parse::uint::<u32>, parse::expect_byte::<b'\n'>
    ) {
        boxes.push((w, l, h));
        input = new_input;
    }

    boxes
}

fn part1(input: &[(u32, u32, u32)]) -> u32 {
    let mut total_area = 0;

    for (w, l, h) in input.iter().cloned() {
        let s1 = l * w;
        let s2 = w * h;
        let s3 = h * l;

        total_area += (2 * s1 + 2 * s2 + 2 * s3) + min(s1, min(s2, s3));
    }

    total_area
}

fn part2(input: &[(u32, u32, u32)]) -> u32 {
    let mut total_length = 0;

    for (w, l, h) in input.iter().cloned() {
        let m1;
        let m2;

        if w <= l {
            m1 = w;
            m2 = if l < h { l } else { h }
        } else {
            m1 = l;
            m2 = if w < h { w } else { h }
        }

        total_length += (m1 * m1 + m2 + m2) + (w * l * h)
    }

    total_length
}
