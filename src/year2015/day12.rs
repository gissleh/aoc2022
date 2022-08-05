use common::parse;
common::day!(parse, part1, part2, 1000, 80, 8);

fn part1(tokens: &[Token]) -> i32 {
    tokens.iter().map(|v| match *v {
        Token::Number(v) => v,
        _ => 0,
    }).sum()
}

fn part2(tokens: &[Token]) -> i32 {
    check_object(tokens).0
}

fn check_object(tokens: &[Token]) -> (i32, usize) {
    let mut count = 0;
    let mut len = 0;
    let mut bad = false;
    let mut array_depth = 0;

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token::ArrayStart => {
                array_depth += 1;
            }
            Token::ArrayEnd => {
                array_depth -= 1;
            }
            Token::Number(v) => {
                count += v;
            }
            Token::String(s) => {
                if s == b"red" && array_depth == 0 {
                    bad = true;
                }
            }
            Token::ObjectStart => {
                let (count2, len2) = check_object(&tokens[i + 1..]);
                i += len2 + 1;
                count += count2;
            }
            Token::ObjectEnd => {
                len = i;
                break;
            }
            _ => {}
        }

        i += 1;
    }

    if bad {
        count = 0;
    }

    (count, len)
}

fn parse(mut input: &[u8]) -> Vec<Token> {
    let mut result = Vec::with_capacity(64);
    while !input.is_empty() {
        // Parse control tokens
        if let Some((b, next)) = parse::byte(input) {
            let mut advance = true;
            match b {
                b'{' => { result.push(Token::ObjectStart) }
                b'}' => { result.push(Token::ObjectEnd) }
                b'[' => { result.push(Token::ArrayStart) }
                b']' => { result.push(Token::ArrayEnd) }
                b':' | b',' | b' ' | b'\n' => {}
                _ => {advance = false}
            }

            if advance {
                input = next;
            }
        }

        // Parse String
        if let Some((_, s, _, next)) = common::parse_all!(
            input,
            parse::expect_byte::<b'"'>,
            parse::until_byte::<b'"'>,
            parse::expect_byte::<b'"'>
        ) {
            input = next;

            // If there's a ':' after, it's a key.
            if let Some((_, next)) = parse::expect_byte::<b':'>(input) {
                result.push(Token::Key(s));
                input = next;
            } else {
                result.push(Token::String(s));
            }
        }

        // Parse number
        if let Some((n, next)) = parse::int::<i32>(input) {
            input = next;
            result.push(Token::Number(n));
        }
    }

    result
}

#[derive(Debug)]
enum Token<'a> {
    ArrayStart,
    ArrayEnd,
    ObjectStart,
    ObjectEnd,
    Key(&'a [u8]),
    Number(i32),
    String(&'a [u8]),
}

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE_1: &'static [u8] = br#"[1,2,3]"#;
    const P1_EXAMPLE_2: &'static [u8] = br#"{"a":2,"b":4}"#;
    const P1_EXAMPLE_3: &'static [u8] = br#"[[[3]]]"#;
    const P1_EXAMPLE_4: &'static [u8] = br#"{"a":{"b":4},"c":-1}"#;
    const P1_EXAMPLE_5: &'static [u8] = br#"{"a":[-1,1]}"#;
    const P1_EXAMPLE_6: &'static [u8] = br#"[-1,{"a":1}]"#;
    const P1_EXAMPLE_7: &'static [u8] = br#"[]"#;
    const P1_EXAMPLE_8: &'static [u8] = br#"{}"#;

    #[test]
    fn p1_counts_correctly() {
        assert_eq!(part1(&parse(P1_EXAMPLE_1)), 6);
        assert_eq!(part1(&parse(P1_EXAMPLE_2)), 6);
        assert_eq!(part1(&parse(P1_EXAMPLE_3)), 3);
        assert_eq!(part1(&parse(P1_EXAMPLE_4)), 3);
        assert_eq!(part1(&parse(P1_EXAMPLE_5)), 0);
        assert_eq!(part1(&parse(P1_EXAMPLE_6)), 0);
        assert_eq!(part1(&parse(P1_EXAMPLE_7)), 0);
        assert_eq!(part1(&parse(P1_EXAMPLE_8)), 0);
    }

    const P2_EXAMPLE_1: &'static [u8] = br#"[1,2,3]"#;
    const P2_EXAMPLE_2: &'static [u8] = br#"[1,{"c":"red","b":2},3]"#;
    const P2_EXAMPLE_3: &'static [u8] = br#"{"d":"red","e":[1,2,3,4],"f":5}"#;
    const P2_EXAMPLE_4: &'static [u8] = br#"[1,"red",5]"#;

    #[test]
    fn p2_counts_reds_correctly() {
        assert_eq!(part2(&parse(P2_EXAMPLE_1)), 6);
        assert_eq!(part2(&parse(P2_EXAMPLE_2)), 4);
        assert_eq!(part2(&parse(P2_EXAMPLE_3)), 0);
        assert_eq!(part2(&parse(P2_EXAMPLE_4)), 6);
    }
}