use common::parse;
common::day!(parse, part1, part2, 1000, 1000, 1000);

fn part1(lines: &[Line]) -> usize {
    let mut raw_total = 0;
    let mut parsed_total = 0;

    for line in lines.iter() {
        for s in line.symbols.iter() {
            parsed_total += match s {
                Symbol::Plain(l) => *l,
                Symbol::Code(_) => 1,
                Symbol::Escaped(_) => 1,
            };
        }

        raw_total += line.data.len();
    }

    raw_total - parsed_total
}

fn part2(lines: &[Line]) -> usize {
    let mut encoded_total = 0;
    let mut raw_total = 0;

    for line in lines.iter() {
        encoded_total += 6;

        for s in line.symbols.iter() {
            encoded_total += match s {
                Symbol::Plain(l) => *l,
                Symbol::Code(_) => 5,
                Symbol::Escaped(_) => 4,
            };
        }

        raw_total += line.data.len();
    }

    encoded_total - raw_total
}

fn parse(mut input: &[u8]) -> Vec<Line> {
    let mut res = Vec::with_capacity(64);

    while let Some((mut line, next)) = parse::line(input) {
        if line.len() == 0 {
            break;
        }

        let mut current = Line{data: line.to_owned(), symbols: Vec::with_capacity(8)};
        while let Some((plain, mut next)) = parse::until_byte::<b'\\'>(line) {
            if plain.len() > 0 {
                current.symbols.push(Symbol::Plain(plain.len()));
            }

            if let Some((_, new_next)) = parse::expect_bytes(b"\\\\")(next) {
                current.symbols.push(Symbol::Escaped(b'\\'));
                next = new_next;
            }
            if let Some((_, new_next)) = parse::expect_bytes(b"\\\"")(next) {
                current.symbols.push(Symbol::Escaped(b'\"'));
                next = new_next;
            }
            if let Some((_, new_next)) = parse::expect_bytes(b"\\x")(next) {
                next = new_next;

                if let Some((byte, new_next)) = parse::hex_byte(next) {
                    next = new_next;
                    current.symbols.push(Symbol::Code(byte))
                } else {
                    current.symbols.push(Symbol::Escaped(b'x'));
                }
            }

            line = next;

            assert!(current.symbols.len() < 1000);
        }

        if let Some(Symbol::Plain(l)) = current.symbols.first_mut() {
            *l -= 1;
        }
        if let Some(Symbol::Plain(l)) = current.symbols.last_mut() {
            *l -= 1;
        }

        res.push(current);
        input = next;

        assert!(res.len() < 10000);
    }

    res
}

#[derive(Eq, PartialEq, Debug)]
struct Line {
    data: Vec<u8>,
    symbols: Vec<Symbol>,
}

#[derive(Eq, PartialEq, Debug)]
enum Symbol {
    Plain(usize),
    Escaped(u8),
    Code(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLES: &'static [u8] = br###"""
"abc"
"aaa\"aaa"
"\x27"
"###;

    const DIFFICULT_INPUT: &'static [u8] = br###""trajs\x5brom\xf1yoijaumkem\"\"tahlzs"
"###;

    #[test]
    fn parse_one_from_my_input() {
        let data = parse(DIFFICULT_INPUT);

        assert_eq!(data.len(), 1);
        assert_eq!(data[0], Line{
            data: br##""trajs\x5brom\xf1yoijaumkem\"\"tahlzs""##.to_vec(),
            symbols: vec![
                Symbol::Plain(5),
                Symbol::Code(0x5b),
                Symbol::Plain(3),
                Symbol::Code(0xf1),
                Symbol::Plain(10),
                Symbol::Escaped(b'\"'),
                Symbol::Escaped(b'\"'),
                Symbol::Plain(6),
            ]
        })
    }

    #[test]
    fn parse_works_with_examples() {
        let data = parse(P1_EXAMPLES);

        assert_eq!(data.len(), 4);
        assert_eq!(data[0], Line{
            data: b"\"\"".to_vec(),
            symbols: vec![
                Symbol::Plain(0)
            ],
        });
        assert_eq!(data[1], Line{
            data: b"\"abc\"".to_vec(),
            symbols: vec![
                Symbol::Plain(3)
            ],
        });
        assert_eq!(data[2], Line{
            data: b"\"aaa\\\"aaa\"".to_vec(),
            symbols: vec![
                Symbol::Plain(3),
                Symbol::Escaped(b'\"'),
                Symbol::Plain(3),
            ],
        });
        assert_eq!(data[3], Line{
            data: b"\"\\x27\"".to_vec(),
            symbols: vec![
                Symbol::Plain(0),
                Symbol::Code(0x27),
                Symbol::Plain(0),
            ],
        });
    }

    #[test]
    fn p1_works_on_example() {
        let data = parse(P1_EXAMPLES);
        assert_eq!(part1(&data), 12);
    }

    #[test]
    fn p2_works_on_example() {
        let data = parse(P1_EXAMPLES);
        assert_eq!(part2(&data), 19);
    }
}