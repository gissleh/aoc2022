use std::ops::Neg;
use num::Integer;
use crate::geo::{Point, Vertex};

#[derive(Eq, PartialEq, Debug)]
pub enum ParseResult<'a, T> {
    Good(T, &'a [u8]),
    Bad,
}

impl<'a, T> ParseResult<'a, T> {
    pub fn or<F>(self, f: F) -> Self where F: FnOnce() -> Self {
        if let Self::Bad = self {
            f()
        } else {
            self
        }
    }

    pub fn and<T2, F>(self, f: F) -> ParseResult<'a, (T, T2)> where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        if let Self::Good(t1, input) = self {
            if let ParseResult::<'a, T2>::Good(t2, input) = f(input) {
                return ParseResult::<'a, (T, T2)>::Good((t1, t2), input);
            }
        }

        ParseResult::<'a, (T, T2)>::Bad
    }

    pub fn and_discard<T2, F>(self, f: F) -> Self where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        if let Self::Good(t1, input) = self {
            if let ParseResult::<'a, T2>::Good(_, input) = f(input) {
                return Self::Good(t1, input);
            }
        }

        Self::Bad
    }

    pub fn and_instead<T2, F>(self, f: F) -> ParseResult<'a, T2> where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        if let Self::Good(_, input) = self {
            if let ParseResult::<'a, T2>::Good(t2, input) = f(input) {
                return ParseResult::<'a, T2>::Good(t2, input);
            }
        }

        ParseResult::<'a, T2>::Bad
    }


    pub fn map<T2, F>(self, f: F) -> ParseResult<'a, T2> where F: FnOnce(T) -> T2 {
        if let Self::Good(t1, input) = self {
            ParseResult::<'a, T2>::Good(f(t1), input)
        } else {
            ParseResult::<'a, T2>::Bad
        }
    }
}

pub fn expect_byte<const P: u8>(input: &[u8]) -> ParseResult<'_, u8> {
    if input.first() == Some(&P) {
        ParseResult::Good(P, &input[1..])
    } else {
        ParseResult::Bad
    }
}

pub fn skip_byte<const P: u8>(input: &[u8]) -> ParseResult<'_, bool> {
    if input.first() == Some(&P) {
        ParseResult::Good(true, &input[1..])
    } else {
        ParseResult::Good(false, input)
    }
}

pub fn expect_bytes<'i, 'p>(pred: &'p [u8]) -> impl Fn(&'i [u8]) -> ParseResult<'i, &'i [u8]> + 'p {
    |input| if input.starts_with(pred) {
        ParseResult::Good(&input[..pred.len()], &input[pred.len()..])
    } else {
        ParseResult::Bad
    }
}

pub fn hex_digit(input: &[u8]) -> ParseResult<u8> {
    if let Some(v) = input.first() {
        match *v {
            b'0'..=b'9' => ParseResult::Good(*v - b'0', &input[1..]),
            b'a'..=b'f' => ParseResult::Good((*v - b'a') + 10, &input[1..]),
            b'A'..=b'F' => ParseResult::Good((*v - b'A') + 10, &input[1..]),
            _ => ParseResult::Bad,
        }
    } else {
        ParseResult::Bad
    }
}

pub fn hex_byte(input: &[u8]) -> ParseResult<u8> {
    hex_digit(input)
        .and(hex_digit)
        .map(|(d1, d2)| d1 * 16 + d2)
        .or(|| hex_digit(input))
}

pub fn int<T: Integer + Copy + From<u8> + Neg<Output=T>>(input: &[u8]) -> ParseResult<'_, T> {
    if input.is_empty() {
        return ParseResult::Bad;
    }

    let mut sum = T::zero();
    let mut neg = false;
    let ten = T::from(10u8);

    for (i, b) in input.iter().enumerate() {
        match *b {
            b'0'..=b'9' => {
                sum = sum.mul(ten);
                sum = sum.add(T::from(*b - b'0'));
            }
            b'-' if !neg => {
                neg = true;
            }
            _ => {
                return if i > 0 {
                    if neg {
                        sum = sum.neg()
                    }
                    ParseResult::Good(sum, &input[i..])
                } else {
                    ParseResult::Bad
                };
            }
        }
    }

    if neg {
        sum = sum.neg()
    }

    ParseResult::Good(sum, &input[input.len()..])
}

pub fn uint<T: Integer + Copy + From<u8>>(input: &[u8]) -> ParseResult<'_, T> {
    if input.is_empty() {
        return ParseResult::Bad;
    }

    let mut sum = T::zero();
    let ten = T::from(10u8);

    for (i, b) in input.iter().enumerate() {
        match *b {
            b'0'..=b'9' => {
                sum = sum.mul(ten);
                sum = sum.add(T::from(*b - b'0'));
            }
            _ => {
                return if i > 0 {
                    ParseResult::Good(sum, &input[i..])
                } else {
                    ParseResult::Bad
                };
            }
        }
    }

    ParseResult::Good(sum, &input[input.len()..])
}

pub fn blank(input: &[u8]) -> ParseResult<()> {
    ParseResult::Good((), input)
}

pub fn point<T, F>(term: F) -> impl Fn(&[u8]) -> ParseResult<'_, Point<T>> where F: Fn(&[u8]) -> ParseResult<T> + Copy, T: Copy {
    move |input| skip_byte::<b'<'>(input)
        .and_instead(term)
        .and_discard(expect_byte::<b','>)
        .and_discard(skip_byte::<b' '>)
        .and(term)
        .and_discard(skip_byte::<b'>'>)
        .map(|(x, y)| Point(x, y))
}

pub fn vertex<T, F>(term: F) -> impl Fn(&[u8]) -> ParseResult<'_, Vertex<T>> where F: Fn(&[u8]) -> ParseResult<T> + Copy, T: Copy {
    move |input| skip_byte::<b'<'>(input)
        .and_instead(term)
        .and_discard(expect_byte::<b','>)
        .and_discard(skip_byte::<b' '>)
        .and(term)
        .and_discard(expect_byte::<b','>)
        .and_discard(skip_byte::<b' '>)
        .and(term)
        .and_discard(skip_byte::<b'>'>)
        .map(|((x, y), z)| Vertex(x, y, z))
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_point() {
        assert_eq!(point(uint::<u32>)(b"<0,14>"), ParseResult::Good(Point(0u32, 14u32), b""));
        assert_eq!(point(int::<i32>)(b"432, -12stuff"), ParseResult::Good(Point(432i32, -12i32), b"stuff"));
        assert_eq!(point(int::<i32>)(b"bad stuff"), ParseResult::Bad);
        assert_eq!(point(int::<i32>)(b"<four, nine>"), ParseResult::Bad);
        assert_eq!(point(int::<i32>)(b"<64.2, 112.23>"), ParseResult::Bad);
    }
}