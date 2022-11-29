pub fn crt(values: &[(i64, i64)]) -> i64 {
    let prod = values.iter()
        .map(|(_, m)| *m as i64)
        .product::<i64>();

    let sum = values.iter()
        .map(|(r, m)| r * mod_inv(prod / *m, *m).unwrap() * (prod / *m))
        .sum::<i64>();

    sum % prod
}

fn egcd(a: i64, b: i64) -> (i64, i64, i64) {
    if a == 0 {
        (b, 0, 1)
    } else {
        let (g, x, y) = egcd(b % a, a);
        (g, y - (b / a) * x, x)
    }
}

fn mod_inv(x: i64, n: i64) -> Option<i64> {
    let (g, x, _) = egcd(x, n);
    if g == 1 {
        Some((x % n + n) % n)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn year2020_day13_shuttle_case() {
        assert_eq!(crt(&[
            (7, 7),
            (12, 13),
            (55, 59),
            (25, 31),
            (12, 19),
        ]), 1068781);
    }

    #[test]
    fn year2020_day13_puzzle_input() {
        assert_eq!(crt(&[
            (41, 41),
            (2, 37),
            (338, 379),
            (20, 23),
            (11, 13),
            (10, 17),
            (17, 29),
            (485, 557),
            (4, 19),
        ]), 840493039281088);
    }

    #[test]
    fn rosetta_example() {
        assert_eq!(crt(&[
            (2, 3), // *id - (minutes % *id)
            (3, 5),
            (2, 7),
        ]), 23);
    }
}
