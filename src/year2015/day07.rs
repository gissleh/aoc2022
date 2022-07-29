use rustc_hash::FxHashMap;
use common::parse;

common::day!(parse, part1, part2, 1000, 100, 100);

fn part1(wires: &[Wire]) -> u16 {
    resolve_all(wires, 0)[0]
}

fn part2(wires: &[Wire]) -> u16 {
    let mut wires2 = wires.to_vec();

    let a = resolve_all(wires, 0)[0];
    wires2[1].input = Signal::DIRECT(Operand::Number(a));

    resolve_all(&wires2, 0)[0]
}

fn resolve_all(wires: &[Wire], stopper: usize) -> Vec<u16> {
    let mut res = vec![0; wires.len()];
    let mut resolved = vec![false; wires.len()];
    let mut remaining = res.len();

    while remaining > 0 {
        for (i, wire) in wires.iter().enumerate() {
            if resolved[i] {
                continue;
            }

            let mut found = false;
            match wire.input {
                Signal::None => {
                    found = true
                }

                Signal::DIRECT(j) => {
                    if let Some(j) = resolve_dependency(&res, &resolved, j) {
                        res[i] = j;
                        found = true
                    }
                }

                Signal::NOT(j) => {
                    if let Some(j) = resolve_dependency(&res, &resolved, j) {
                        res[i] = !j;
                        found = true
                    }
                }

                Signal::AND(j, k) => {
                    if let Some((j, k)) = resolve_dependencies(&res, &resolved, j, k) {
                        res[i] = j & k;
                        found = true
                    }
                }

                Signal::OR(j, k) => {
                    if let Some((j, k)) = resolve_dependencies(&res, &resolved, j, k) {
                        res[i] = j | k;
                        found = true
                    }
                }

                Signal::LSHIFT(j, k) => {
                    if let Some((j, k)) = resolve_dependencies(&res, &resolved, j, k) {
                        res[i] = j << k;
                        found = true
                    }
                }

                Signal::RSHIFT(j, k) => {
                    if let Some((j, k)) = resolve_dependencies(&res, &resolved, j, k) {
                        res[i] = j >> k;
                        found = true
                    }
                }
            }

            if found {
                if i == stopper {
                    return res;
                }

                resolved[i] = true;
                remaining -= 1;
            }
        }
    }


    res
}

fn resolve_dependency(res: &[u16], resolved: &[bool], op: Operand) -> Option<u16> {
    match op {
        Operand::Number(n) => Some(n),
        Operand::Wire(j) => {
            if resolved[j] {
                Some(res[j])
            } else {
                None
            }
        }
    }
}

fn resolve_dependencies(res: &[u16], resolved: &[bool], op_1: Operand, op_2: Operand) -> Option<(u16, u16)> {
    if let Some(v1) = resolve_dependency(res, resolved, op_1) {
        if let Some(v2) = resolve_dependency(res, resolved, op_2) {
            return Some((v1, v2));
        }
    }

    None
}

fn parse(input: &[u8]) -> Vec<Wire> {
    let mut input = input;
    let mut wires: Vec<Wire> = Vec::with_capacity(64);
    let mut map: FxHashMap<Vec<u8>, usize> = FxHashMap::default();

    // Hard-code a and b to be at the start
    wires.push(Wire { input: Signal::None });
    wires.push(Wire { input: Signal::None });
    map.insert(b"a".to_vec(), 0);
    map.insert(b"b".to_vec(), 1);

    while let Some((mut line, next)) = parse::line(input) {
        if line.is_empty() {
            break;
        }

        let mut parse_state = 0;
        let mut left: &[u8] = &line[..0];
        let mut left_number = 0u16;
        let mut right: &[u8] = &line[..0];
        let mut right_number = 0u16;
        let mut op: &[u8] = &line[..0];
        let mut dst: &[u8] = &line[..0];

        // Find the pieces
        while let Some((word, next)) = parse::word(line) {
            match parse_state {
                0 => {
                    if word[0].is_ascii_uppercase() {
                        op = word;
                        parse_state = 2;
                    } else if word[0].is_ascii_digit() {
                        let (n, _) = parse::uint(word).unwrap();
                        left_number = n;
                        left = word;
                        parse_state = 1;
                    } else {
                        left = word;
                        parse_state = 1;
                    }
                }

                1 => {
                    if word[0].is_ascii_uppercase() {
                        op = word;
                        parse_state = 2;
                    } else if word == b"->" {
                        parse_state = 3;
                    } else {
                        panic!("unknown line: {}", String::from_utf8_lossy(line));
                    }
                }

                2 => {
                    if word[0].is_ascii_alphabetic() {
                        right = word;
                    } else {
                        let (n, _) = parse::uint(word).unwrap();
                        right_number = n;
                        right = word;
                    }
                    parse_state = 3;
                }

                3 => {
                    if word != b"->" {
                        dst = word;
                        parse_state = 4;
                    }
                }

                _ => {}
            }

            line = next;
        }
        assert_eq!(parse_state, 4);

        // Add wires to list
        let dst_index = ensure_wire(&mut wires, &mut map, dst);
        let left_op = if left_number != 0 {
            Operand::Number(left_number)
        } else {
            Operand::Wire(ensure_wire(&mut wires, &mut map, left))
        };
        let right_op = if right_number != 0 {
            Operand::Number(right_number)
        } else {
            Operand::Wire(ensure_wire(&mut wires, &mut map, right))
        };

        match op {
            b"NOT" => {
                wires[dst_index] = Wire { input: Signal::NOT(right_op) }
            }
            b"AND" => {
                wires[dst_index] = Wire { input: Signal::AND(left_op, right_op) }
            }
            b"OR" => {
                wires[dst_index] = Wire { input: Signal::OR(left_op, right_op) }
            }
            b"LSHIFT" => {
                wires[dst_index] = Wire { input: Signal::LSHIFT(left_op, right_op) }
            }
            b"RSHIFT" => {
                wires[dst_index] = Wire { input: Signal::RSHIFT(left_op, right_op) }
            }
            b"" => {
                wires[dst_index] = Wire { input: Signal::DIRECT(left_op) }
            }

            _ => unreachable!()
        }

        input = next
    }

    wires
}

fn ensure_wire(wires: &mut Vec<Wire>, map: &mut FxHashMap<Vec<u8>, usize>, key: &[u8]) -> usize {
    if key.is_empty() {
        return usize::MAX;
    }

    return if let Some(existing_index) = map.get(key) {
        *existing_index
    } else {
        let new_index = wires.len();
        wires.push(Wire { input: Signal::None });
        map.insert(Vec::from(key), new_index);

        new_index
    };
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Wire {
    input: Signal,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Signal {
    None,
    DIRECT(Operand),
    AND(Operand, Operand),
    OR(Operand, Operand),
    NOT(Operand),
    LSHIFT(Operand, Operand),
    RSHIFT(Operand, Operand),
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Operand {
    Number(u16),
    Wire(usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE_01: &[u8] = b"123 -> x
456 -> y
x AND y -> d
x OR y -> e
x LSHIFT 2 -> f
y RSHIFT 2 -> g
NOT x -> h
NOT y -> i
i -> z
";

    #[test]
    fn test_parse_example() {
        let wires = parse(EXAMPLE_01);

        assert_eq!(wires, &[
            Wire { input: Signal::None },
            Wire { input: Signal::None },
            Wire { input: Signal::DIRECT(Operand::Number(123)) },
            Wire { input: Signal::DIRECT(Operand::Number(456)) },
            Wire { input: Signal::AND(Operand::Wire(2), Operand::Wire(3)) },
            Wire { input: Signal::OR(Operand::Wire(2), Operand::Wire(3)) },
            Wire { input: Signal::LSHIFT(Operand::Wire(2), Operand::Number(2)) },
            Wire { input: Signal::RSHIFT(Operand::Wire(3), Operand::Number(2)) },
            Wire { input: Signal::NOT(Operand::Wire(2)) },
            Wire { input: Signal::NOT(Operand::Wire(3)) },
            Wire { input: Signal::DIRECT(Operand::Wire(9)) },
        ]);
    }

    #[test]
    fn test_run_example() {
        let wires = parse(EXAMPLE_01);
        assert_eq!(resolve_all(&wires, 1000), &[
            0,
            0,
            123,
            456,
            72,
            507,
            492,
            114,
            65412,
            65079,
            65079,
        ]);
    }
}