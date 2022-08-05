use std::mem;
common::day!(parse, part1, part2, 1000, 80, 8);

fn part1(input: &[u8]) -> usize {
    resolve(input, 40).len()
}

fn part2(input: &[u8]) -> usize {
    resolve(input, 50).len()
}

fn parse(input: &[u8]) -> Vec<u8> {
    input.iter().take_while(|v| **v != b'\n').map(|v| (*v - b'0') as u8).collect()
}

fn resolve(input: &[u8], steps: u32) -> Vec<u8> {
    let mut curr = Vec::with_capacity(1048576);
    let mut next = Vec::with_capacity(1048576);

    curr.extend_from_slice(&input);

    for _ in 0..steps {
        step(&curr, &mut next);
        mem::swap(&mut curr, &mut next);
    }

    curr
}

fn step(src: &[u8], dst: &mut Vec<u8>) {
    let mut prev = 0u8;
    let mut count = 0u32;

    dst.clear();

    for n in src.iter().cloned() {
        if n == prev {
            count += 1;
        } else if count > 0 {
            assert!(count < 10);

            dst.push(count as u8);
            dst.push(prev);
            count = 1;
        } else {
            count = 1;
        }

        prev = n;
    }

    if count > 0 {
        dst.push(count as u8);
        dst.push(prev);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE_1: &'static [u8] = &[1];
    const P1_EXAMPLE_2: &'static [u8] = &[1,1];
    const P1_EXAMPLE_3: &'static [u8] = &[2,1];
    const P1_EXAMPLE_4: &'static [u8] = &[1,2,1,1];
    const P1_EXAMPLE_5: &'static [u8] = &[1,1,1,2,2,1];
    const P1_EXAMPLE_6: &'static [u8] = &[3,1,2,2,1,1];

    #[test]
    fn step_works_on_example() {
        let mut buf = Vec::with_capacity(16);

        step(P1_EXAMPLE_1, &mut buf);
        assert_eq!(buf.as_slice(), P1_EXAMPLE_2);
        step(P1_EXAMPLE_2, &mut buf);
        assert_eq!(buf.as_slice(), P1_EXAMPLE_3);
        step(P1_EXAMPLE_3, &mut buf);
        assert_eq!(buf.as_slice(), P1_EXAMPLE_4);
        step(P1_EXAMPLE_4, &mut buf);
        assert_eq!(buf.as_slice(), P1_EXAMPLE_5);
        step(P1_EXAMPLE_5, &mut buf);
        assert_eq!(buf.as_slice(), P1_EXAMPLE_6);
    }

    #[test]
    fn resolves_uses_steps_correctly() {
        assert_eq!(resolve(P1_EXAMPLE_1, 4).as_slice(), P1_EXAMPLE_5);
        assert_eq!(resolve(P1_EXAMPLE_2, 3).as_slice(), P1_EXAMPLE_5);
        assert_eq!(resolve(P1_EXAMPLE_1, 5).as_slice(), P1_EXAMPLE_6);
    }
}