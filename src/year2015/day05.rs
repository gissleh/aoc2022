use common::parse;

common::day!(parse, part1, part2, 10000, 10, 10);

fn parse(data: &[u8]) -> Vec<[u8; 16]> {
    let mut words = Vec::with_capacity(64);
    let mut input = data;
    while let Some((word, _, new_input)) = common::parse_all!(
        input,
        parse::byte_array::<16>,
        parse::expect_byte::<b'\n'>
    ) {
        words.push(word);
        input = new_input;
    }

    words
}

fn part1(input: &[[u8; 16]]) -> usize {
    input.iter().copied().filter(is_nice).count()
}

fn part2(input: &[[u8; 16]]) -> usize {
    input.iter().copied().filter(is_really_nice).count()
}

fn is_nice(word: &[u8; 16]) -> bool {
    let has_pair = word.array_windows::<2>().find(|p| p[0] == p[1]).is_some();
    if !has_pair {
        return false;
    }

    let vowels = word.iter().filter(|b| match **b {
        b'a' | b'e' | b'i' | b'o' | b'u' => true,
        _ => false,
    }).count();
    if vowels < 3 {
        return false;
    }

    let has_evil = word.array_windows::<2>().find(|p| match *p {
        b"ab" | b"cd" | b"pq" | b"xy" => true,
        _ => false,
    }).is_some();
    if has_evil {
        return false;
    }

    true
}

fn is_really_nice(word: &[u8; 16]) -> bool {
    let has_xyx = word.array_windows::<3>().find(|a| a[0] == a[2]).is_some();
    if !has_xyx {
        return false;
    }

    for (i, pair) in word.array_windows::<2>().enumerate() {
        for pair2 in word[i+2..].array_windows::<2>() {
            if pair == pair2 {
                return true;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_nice_identifies_correctly() {
        assert_eq!(is_nice(b"ugknbfddgicrmopn"), true);
        assert_eq!(is_nice(b"ugknbfdzgicrmopn"), false);
        assert_eq!(is_nice(b"ugknbfdzgicrmzpn"), false);
        assert_eq!(is_nice(b"jchzalrnumimnmhp"), false);
        assert_eq!(is_nice(b"haegwjzuvuyypabu"), false);
        assert_eq!(is_nice(b"haegwjzuvuyypcdu"), false);
        assert_eq!(is_nice(b"haegwjzuvuyyppqu"), false);
        assert_eq!(is_nice(b"haegwjzuvuyypxyu"), false);
        assert_eq!(is_nice(b"haegwjzuvuyypxzu"), true);
        assert_eq!(is_nice(b"dvszwmarrgswjxmb"), false);
    }

    #[test]
    fn is_really_nice_identifies_correctly() {
        assert_eq!(is_really_nice(b"qjhvhtzxzqqjkmpb"), true);
        assert_eq!(is_really_nice(b"uurcxstgmygtbstg"), false);
        assert_eq!(is_really_nice(b"ieodomkazucvgmuy"), false);
    }

    #[test]
    fn part1_counts_right() {
        assert_eq!(part1(&[
            *b"ugknbfddgicrmopn",
            *b"ugknbfdzgicrmzpn",
            *b"jchzalrnumimnmhp",
            *b"haegwjzuvuyypxyu",
            *b"haegwjzuvuyypxzu",
        ]), 2);
    }

    #[test]
    fn part2_counts_right() {
        assert_eq!(part2(&[
            *b"qjhvhtzxzqqjkmpb",
            *b"uurcxstgmygtbstg",
            *b"ieodomkazucvgmuy",
        ]), 1);
    }
}