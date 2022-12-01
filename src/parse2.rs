use std::ops::Neg;
use num::Integer;
use crate::geo::{Point, Vertex};

#[derive(Eq, PartialEq, Debug)]
pub enum ParseResult<'a, T> {
    Good(T, &'a [u8]),
    Bad(&'a [u8]),
}

impl<'a, T> Into<Option<T>> for ParseResult<'a, T> {
    fn into(self) -> Option<T> {
        match self {
            Self::Good(v, _) => Some(v),
            Self::Bad(_) => None,
        }
    }
}

impl<'a, T> ParseResult<'a, T> {
    pub fn is_good(&self) -> bool {
        match self {
            Self::Good(_, _) => true,
            Self::Bad(_) => false,
        }
    }

    pub fn is_bad(&self) -> bool {
        match self {
            Self::Good(_, _) => false,
            Self::Bad(_) => true,
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Good(t, _) => t,
            Self::Bad(_) => panic!("unwrap on a bad parse result"),
        }
    }

    pub fn or<F>(self, f: F) -> Self where F: FnOnce(&'a [u8]) -> ParseResult<'a, T> {
        if let Self::Bad(input) = self {
            f(input)
        } else {
            self
        }
    }

    pub fn and<T2, F>(self, f: F) -> ParseResult<'a, (T, T2)> where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        match self {
            Self::Good(t1, input) => {
                return if let ParseResult::<'a, T2>::Good(t2, input) = f(input) {
                    ParseResult::<'a, (T, T2)>::Good((t1, t2), input)
                } else {
                    ParseResult::<'a, (T, T2)>::Bad(input)
                };
            }
            Self::Bad(input) => ParseResult::<'a, (T, T2)>::Bad(input)
        }
    }

    pub fn and_discard<T2, F>(self, f: F) -> Self where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        match self {
            Self::Good(t1, input) => {
                return if let ParseResult::<'a, T2>::Good(_, input) = f(input) {
                    Self::Good(t1, input)
                } else {
                    Self::Bad(input)
                };
            }
            v => v
        }
    }

    pub fn and_instead<T2, F>(self, f: F) -> ParseResult<'a, T2> where F: FnOnce(&'a [u8]) -> ParseResult<'a, T2> {
        match self {
            Self::Good(_, input) => {
                return if let ParseResult::<'a, T2>::Good(t2, input) = f(input) {
                    return ParseResult::<'a, T2>::Good(t2, input);
                } else {
                    ParseResult::<'a, T2>::Bad(input)
                };
            }
            Self::Bad(input) => ParseResult::<'a, T2>::Bad(input)
        }
    }

    pub fn map<T2, F>(self, f: F) -> ParseResult<'a, T2> where F: FnOnce(T) -> T2 {
        match self {
            Self::Good(t1, input) => ParseResult::<'a, T2>::Good(f(t1), input),
            Self::Bad(input) => ParseResult::<'a, T2>::Bad(input)
        }
    }
}

pub fn paragraph(input: &[u8]) -> ParseResult<'_, &[u8]> {
    if input.len() < 2 {
        return ParseResult::Bad(input);
    }

    let line_len = input.windows(2).take_while(|v| *v != b"\n\n").count();
    if line_len == input.len() {
        ParseResult::Good(&input[..line_len], &input[line_len..])
    } else if line_len == input.len() - 1 {
        ParseResult::Good(&input[..line_len], &input[line_len + 1..])
    } else {
        ParseResult::Good(&input[..line_len], &input[line_len + 2..])
    }
}


pub fn line(input: &[u8]) -> ParseResult<'_, &[u8]> {
    if input.len() == 0 {
        return ParseResult::Bad(input);
    }

    let line_len = input.iter().take_while(|v| **v != b'\n').count();
    if line_len == input.len() {
        ParseResult::Good(&input[..line_len], &input[line_len..])
    } else {
        ParseResult::Good(&input[..line_len], &input[line_len + 1..])
    }
}

pub fn n_bytes<const N: usize>(input: &[u8]) -> ParseResult<'_, &[u8]> {
    if input.len() >= N {
        ParseResult::Good(&input[..N], &input[N..])
    } else {
        ParseResult::Bad(input)
    }
}

pub fn expect_byte<const P: u8>(input: &[u8]) -> ParseResult<'_, u8> {
    if input.first() == Some(&P) {
        ParseResult::Good(P, &input[1..])
    } else {
        ParseResult::Bad(input)
    }
}

pub fn skip_byte<const P: u8>(input: &[u8]) -> ParseResult<'_, bool> {
    if input.first() == Some(&P) {
        ParseResult::Good(true, &input[1..])
    } else {
        ParseResult::Good(false, input)
    }
}

pub fn skip_all_bytes<const P: u8>(input: &[u8]) -> ParseResult<'_, usize> {
    let count = input.iter().take_while(|v| P.eq(*v)).count();
    ParseResult::Good(count, &input[count..])
}

pub fn expect_bytes<'i, 'p>(pred: &'p [u8]) -> impl Fn(&'i [u8]) -> ParseResult<'i, &'i [u8]> + 'p {
    |input| if input.starts_with(pred) {
        ParseResult::Good(&input[..pred.len()], &input[pred.len()..])
    } else {
        ParseResult::Bad(input)
    }
}

pub fn hex_digit(input: &[u8]) -> ParseResult<u8> {
    if let Some(v) = input.first() {
        match *v {
            b'0'..=b'9' => ParseResult::Good(*v - b'0', &input[1..]),
            b'a'..=b'f' => ParseResult::Good((*v - b'a') + 10, &input[1..]),
            b'A'..=b'F' => ParseResult::Good((*v - b'A') + 10, &input[1..]),
            _ => ParseResult::Bad(input),
        }
    } else {
        ParseResult::Bad(input)
    }
}

pub fn hex_byte(input: &[u8]) -> ParseResult<u8> {
    hex_digit(input)
        .and(hex_digit)
        .map(|(d1, d2)| d1 * 16 + d2)
        .or(hex_digit)
}

pub fn int<T: Integer + Copy + From<u8> + Neg<Output=T>>(input: &[u8]) -> ParseResult<'_, T> {
    if input.is_empty() {
        return ParseResult::Bad(input);
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
                    ParseResult::Bad(input)
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
        return ParseResult::Bad(input);
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
                    ParseResult::Bad(input)
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
        .and_discard(skip_all_bytes::<b' '>)
        .and(term)
        .and_discard(skip_byte::<b'>'>)
        .map(|(x, y)| Point(x, y))
}

pub fn vertex<T, F>(term: F) -> impl Fn(&[u8]) -> ParseResult<'_, Vertex<T>> where F: Fn(&[u8]) -> ParseResult<T> + Copy, T: Copy {
    move |input| skip_byte::<b'<'>(input)
        .and_instead(term)
        .and_discard(expect_byte::<b','>)
        .and_discard(skip_all_bytes::<b' '>)
        .and(term)
        .and_discard(expect_byte::<b','>)
        .and_discard(skip_all_bytes::<b' '>)
        .and(term)
        .and_discard(skip_byte::<b'>'>)
        .map(|((x, y), z)| Vertex(x, y, z))
}

pub struct Map<'a, T, F> {
    input: &'a [u8],
    f: F,
    spooky_ghost: std::marker::PhantomData<T>,
}

impl<'a, T, F> Iterator for Map<'a, T, F> where F: Fn(&'a [u8]) -> ParseResult<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if let ParseResult::Good(value, new_input) = (self.f)(self.input) {
            self.input = new_input;

            Some(value)
        } else {
            None
        }
    }
}

pub fn map<'a, T, F>(input: &'a [u8], f: F) -> Map<'a, T, F>
    where F: Fn(&'a [u8]) -> ParseResult<'a, T> {
    Map { input, f, spooky_ghost: std::marker::PhantomData }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_parse_point() {
        assert_eq!(point(uint::<u32>)(b"<0,14>"), ParseResult::Good(Point(0u32, 14u32), b""));
        assert_eq!(point(int::<i32>)(b"432, -12stuff"), ParseResult::Good(Point(432i32, -12i32), b"stuff"));
        assert!(point(int::<i32>)(b"bad stuff").is_bad());
        assert!(point(int::<i32>)(b"<four, nine>").is_bad());
        assert!(point(int::<i32>)(b"<64.2, 112.23>").is_bad());
        assert_eq!(point(int::<i32>)(b"<-117,  16>"), ParseResult::Good(Point(-117i32, 16i32), b""));
    }

    #[test]
    fn test_parse_vertex() {
        assert_eq!(vertex(uint::<u32>)(b"<18,15,13>"), ParseResult::Good(Vertex(18u32, 15u32, 13u32), b""));
        assert_eq!(vertex(int::<i32>)(b"432, -12, 99912stuff"), ParseResult::Good(Vertex(432i32, -12i32, 99912i32), b"stuff"));
        assert!(vertex(int::<i32>)(b"bad stuff, but in 3d").is_bad());
        assert!(vertex(int::<i32>)(b"<four, nine, six>").is_bad());
        assert!(vertex(int::<i32>)(b"<64.2, 112.23, 119.97>").is_bad());
        assert_eq!(vertex(int::<i32>)(b"<-117,  16,   189>"), ParseResult::Good(Vertex(-117i32, 16i32, 189i32), b""));
    }

    #[test]
    fn test_map() {
        let list: Vec<Point<i32>> = map(
            b"<12, -16>\n<19, 23>\n<-112, 12>",
            |input| point(int::<i32>)(input).and_discard(skip_byte::<b'\n'>),
        ).collect();

        assert_eq!(list, vec![Point(12i32, -16i32), Point(19i32, 23i32), Point(-112i32, 12i32)]);
    }
}