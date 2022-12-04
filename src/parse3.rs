use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::Neg;
use num::Integer;
use crate::geo::{Point, Vertex};

pub trait Parser<'i, T> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T>;

    fn and<T2, P2: Parser<'i, T2> + Sized>(self, p2: P2) -> And<'i, T, T2, Self, P2> where Self: Sized {
        And(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn and_instead<T2, P2: Parser<'i, T2> + Sized>(self, p2: P2) -> AndInstead<'i, T, T2, Self, P2> where Self: Sized {
        AndInstead(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn and_discard<T2, P2: Parser<'i, T2> + Sized>(self, p2: P2) -> AndDiscard<'i, T, T2, Self, P2> where Self: Sized {
        AndDiscard(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn skip<T2, P2: Parser<'i, T2> + Sized>(self, p2: P2) -> Skip<'i, T, T2, Self, P2> where Self: Sized {
        Skip(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn or<P2: Parser<'i, T> + Sized>(self, p2: P2) -> Or<'i, T, Self, P2> where Self: Sized {
        Or(self, p2, PhantomData::default())
    }

    fn repeat(self) -> Repeat<'i, T, Self> where Self: Sized {
        Repeat(self, PhantomData::default())
    }

    fn repeat_n(self, n: usize) -> RepeatN<'i, T, Self> where Self: Sized {
        RepeatN(n, self, PhantomData::default())
    }

    fn repeat_arr<const N: usize>(self) -> RepeatArray<'i, N, T, Self> where T: Copy + Default, Self: Sized {
        RepeatArray(self, PhantomData::default())
    }

    fn repeat_until<TE, PE: Parser<'i, TE>>(self, ending: PE) -> RepeatUntil<'i, T, TE, Self, PE> where Self: Sized {
        RepeatUntil(self, ending, PhantomData::default(), PhantomData::default())
    }

    fn quoted_by(self, prefix: u8, suffix: u8) -> Quoted<'i, T, Self> where Self: Sized {
        Quoted(self, prefix, suffix, PhantomData::default())
    }

    fn map<T2, F: Fn(T) -> T2>(self, cb: F) -> Map<'i, T, T2, F, Self> where Self: Sized {
        Map(self, cb, PhantomData::default(), PhantomData::default())
    }

    fn filter<F: Fn(&T) -> bool>(self, cb: F) -> Filter<'i, T, F, Self> where Self: Sized {
        Filter(self, cb, PhantomData::default())
    }
}

pub struct And<'i, T1, T2, P1, P2> (P1, P2, PhantomData<&'i T1>, PhantomData<T2>) where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized;

impl<'i, T1, T2, P1, P2> Parser<'i, (T1, T2)> for And<'i, T1, T2, P1, P2> where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, (T1, T2)> {
        if let ParseResult::Good(t1, input) = self.0.parse(input) {
            if let ParseResult::Good(t2, input) = self.1.parse(input) {
                return ParseResult::Good((t1, t2), input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct AndDiscard<'i, T1, T2, P1, P2> (P1, P2, PhantomData<&'i T1>, PhantomData<T2>) where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized;

impl<'i, T1, T2, P1, P2> Parser<'i, T1> for AndDiscard<'i, T1, T2, P1, P2> where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T1> {
        if let ParseResult::Good(t1, input) = self.0.parse(input) {
            if let ParseResult::Good(_, input) = self.1.parse(input) {
                return ParseResult::Good(t1, input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct Skip<'i, T1, T2, P1, P2> (P1, P2, PhantomData<&'i T1>, PhantomData<T2>) where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized;

impl<'i, T1, T2, P1, P2> Parser<'i, T1> for Skip<'i, T1, T2, P1, P2> where P1: Parser<'i, T1> + Sized, P2: Parser<'i, T2> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T1> {
        if let ParseResult::Good(t1, input) = self.0.parse(input) {
            return if let ParseResult::Good(_, input) = self.1.parse(input) {
                ParseResult::Good(t1, input)
            } else {
                ParseResult::Good(t1, input)
            };
        }

        ParseResult::Bad(input)
    }
}

pub struct AndInstead<'i, T1, T2, P1, P2> (P1, P2, PhantomData<&'i T1>, PhantomData<T2>) where P1: Parser<'i, T1>, P2: Parser<'i, T2>;

impl<'i, T1, T2, P1, P2> Parser<'i, T2> for AndInstead<'i, T1, T2, P1, P2> where P1: Parser<'i, T1>, P2: Parser<'i, T2> {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T2> {
        if let ParseResult::Good(_, input) = self.0.parse(input) {
            if let ParseResult::Good(t2, input) = self.1.parse(input) {
                return ParseResult::Good(t2, input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct Map<'i, T1, T2, F, P1> (P1, F, PhantomData<&'i T1>, PhantomData<T2>)
    where P1: Parser<'i, T1> + Sized,
          F: Fn(T1) -> T2;

impl<'i, T1, T2, F, P1> Parser<'i, T2> for Map<'i, T1, T2, F, P1>
    where P1: Parser<'i, T1> + Sized,
          F: Fn(T1) -> T2 {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T2> {
        match self.0.parse(input) {
            ParseResult::Good(t1, input) => ParseResult::Good(self.1(t1), input),
            ParseResult::Bad(input) => ParseResult::Bad(input),
        }
    }
}

pub struct Filter<'i, T, F, P> (P, F, PhantomData<&'i T>)
    where P: Parser<'i, T> + Sized,
          F: Fn(&T) -> bool;

impl<'i, T, F, P> Parser<'i, T> for Filter<'i, T, F, P>
    where P: Parser<'i, T> + Sized,
          F: Fn(&T) -> bool {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        match self.0.parse(input) {
            ParseResult::Good(t1, input) => if self.1(&t1) {
                ParseResult::Good(t1, input)
            } else {
                ParseResult::Bad(input)
            },
            ParseResult::Bad(input) => ParseResult::Bad(input),
        }
    }
}

pub struct Or<'i, T, P1, P2> (P1, P2, PhantomData<&'i T>) where P1: Parser<'i, T> + Sized, P2: Parser<'i, T> + Sized;

impl<'i, T, P1, P2> Parser<'i, T> for Or<'i, T, P1, P2> where P1: Parser<'i, T> + Sized, P2: Parser<'i, T> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(t, input) = self.0.parse(input) {
            ParseResult::Good(t, input)
        } else if let ParseResult::Good(t, input) = self.1.parse(input) {
            ParseResult::Good(t, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

pub struct Quoted<'i, T, P> (P, u8, u8, PhantomData<&'i T>) where P: Parser<'i, T> + Sized;

impl<'i, T, P> Parser<'i, T> for Quoted<'i, T, P> where P: Parser<'i, T> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if input.len() == 0 || input[0] != self.1 {
            return ParseResult::Bad(input);
        }

        match self.0.parse(&input[1..]) {
            ParseResult::Good(t, new_input) => {
                if new_input.len() == 0 || new_input[0] != self.2 {
                    ParseResult::Bad(input)
                } else {
                    ParseResult::Good(t, &new_input[1..])
                }
            }
            ParseResult::Bad(_) => ParseResult::Bad(input),
        }
    }
}

pub struct RepeatN<'i, T, P> (usize, P, PhantomData<&'i T>) where P: Parser<'i, T> + Sized;

impl<'i, T, P> Parser<'i, Vec<T>> for RepeatN<'i, T, P> where P: Parser<'i, T> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Vec<T>> {
        let mut current_input = input;
        let mut res = Vec::with_capacity(self.0);

        for _ in 0..self.0 {
            match self.1.parse(current_input) {
                ParseResult::Good(t, new_input) => {
                    res.push(t);
                    current_input = new_input;
                }
                ParseResult::Bad(_) => {
                    return ParseResult::Bad(input);
                }
            }
        }

        ParseResult::Good(res, current_input)
    }
}

pub struct Repeat<'i, T, P> (P, PhantomData<&'i T>) where P: Parser<'i, T> + Sized;

pub struct RepeatArray<'i, const N: usize, T, P> (P, PhantomData<&'i T>) where T: Default + Copy, P: Parser<'i, T> + Sized;

impl<'i, const N: usize, T, P> Parser<'i, [T; N]> for RepeatArray<'i, N, T, P> where T: Default + Copy, P: Parser<'i, T> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, [T; N]> {
        let mut current_input = input;
        let mut res = [T::default(); N];

        for i in 0..N {
            match self.0.parse(current_input) {
                ParseResult::Good(t, new_input) => {
                    res[i] = t;
                    current_input = new_input;
                }
                ParseResult::Bad(_) => {
                    return ParseResult::Bad(input);
                }
            }
        }

        ParseResult::Good(res, current_input)
    }
}

impl<'i, T, P> Parser<'i, Vec<T>> for Repeat<'i, T, P> where P: Parser<'i, T> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Vec<T>> {
        let mut current_input = input;
        let mut res = Vec::with_capacity(64);

        loop {
            match self.0.parse(current_input) {
                ParseResult::Good(t, new_input) => {
                    res.push(t);
                    current_input = new_input;
                }
                ParseResult::Bad(_) => {
                    if res.len() == 0 {
                        return ParseResult::Bad(input);
                    }

                    break;
                }
            }
        }

        ParseResult::Good(res, current_input)
    }
}

pub struct RepeatUntil<'i, TI, TE, PI, PE> (PI, PE, PhantomData<&'i TI>, PhantomData<TE>) where PI: Parser<'i, TI> + Sized, PE: Parser<'i, TE> + Sized;

impl<'i, TI, TE, PI, PE> Parser<'i, Vec<TI>> for RepeatUntil<'i, TI, TE, PI, PE> where PI: Parser<'i, TI> + Sized, PE: Parser<'i, TE> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Vec<TI>> {
        let mut current_input = input;
        let mut res = Vec::with_capacity(64);

        loop {
            match self.0.parse(current_input) {
                ParseResult::Good(t, new_input) => {
                    res.push(t);
                    current_input = new_input;

                    if let ParseResult::Good(_, new_input) = self.1.parse(current_input) {
                        return ParseResult::Good(res, new_input);
                    }
                }
                ParseResult::Bad(_) => {
                    return ParseResult::Bad(input);
                }
            }
        }
    }
}

#[derive(Eq, PartialEq)]
pub enum ParseResult<'i, T> {
    Good(T, &'i [u8]),
    Bad(&'i [u8]),
}

impl<'i, T> ParseResult<'i, T> {
    #[inline]
    pub fn unwrap(self) -> T {
        match self {
            ParseResult::Good(v, _) => v,
            ParseResult::Bad(_) => panic!("unwrap on ParseResult::Bad"),
        }
    }
}

impl<'i, T> Debug for ParseResult<'i, T> where T: Debug {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseResult::Good(v, i) => write!(f, "Good({:?}, {} remaining)", v, i.len()),
            ParseResult::Bad(i) => write!(f, "Bad({} remaining)", i.len())
        }
    }
}

impl<'i, T> Display for ParseResult<'i, T> where T: Display {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseResult::Good(v, _) => write!(f, "Good({})", v),
            ParseResult::Bad(_) => write!(f, "Bad")
        }
    }
}

struct ExpectByte(u8);

impl<'i> Parser<'i, u8> for ExpectByte {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        match input.first() {
            Some(v) if *v == self.0 => ParseResult::Good(*v, &input[1..]),
            _ => ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn expect_byte<'i>(byte: u8) -> impl Parser<'i, u8> {
    ExpectByte(byte)
}

struct AnyByte;

impl<'i> Parser<'i, u8> for AnyByte {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        match input.first() {
            Some(v) => ParseResult::Good(*v, &input[1..]),
            _ => ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn any_byte<'i>() -> impl Parser<'i, u8> {
    AnyByte
}

struct ExpectBytes(&'static [u8]);

impl<'i> Parser<'i, &'i [u8]> for ExpectBytes {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'i [u8]> {
        if self.0.len() == input.len() {
            ParseResult::Bad(input)
        } else if input[..self.0.len()].eq(self.0) {
            ParseResult::Good(&input[..self.0.len()], &input[self.0.len()..])
        } else {
            ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn expect_bytes<'i>(bytes: &'static [u8]) -> impl Parser<'i, &'i [u8]> {
    ExpectBytes(bytes)
}

struct ExpectEitherByte<'p>(&'p [u8]);

impl<'i, 'p> Parser<'i, u8> for ExpectEitherByte<'p> {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        if input.len() == 0 {
            return ParseResult::Bad(input);
        }

        let v = input.first().unwrap();
        for w in self.0.iter() {
            if v.eq(w) {
                return ParseResult::Good(*v, &input[1..]);
            }
        }

        ParseResult::Bad(input)
    }
}

#[inline]
pub fn expect_either_bytes<'i, 'p>(bytes: &'p [u8]) -> impl Parser<'i, u8> + '_ {
    ExpectEitherByte(bytes)
}

struct UnsignedInt<T: Integer + Copy + From<u8>> (PhantomData<T>);

impl<'i, T: Integer + Copy + From<u8>> Parser<'i, T> for UnsignedInt<T> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
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
}

#[inline]
pub fn unsigned_int<'i, T: Integer + Copy + From<u8> + Default>() -> impl Parser<'i, T> {
    return UnsignedInt(PhantomData::default());
}

struct SignedInt<T: Integer + Copy + From<u8> + Neg<Output=T>> (PhantomData<T>);

impl<'i, T: Integer + Copy + From<u8> + Neg<Output=T>> Parser<'i, T> for SignedInt<T> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
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
                b'-' if !neg && i == 0 => {
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
}

#[inline]
pub fn signed_int<'i, T: Integer + Copy + From<u8> + Neg<Output=T>>() -> impl Parser<'i, T> {
    return SignedInt(PhantomData::default());
}

struct Line;

impl<'i> Parser<'i, &'i [u8]> for Line {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'i [u8]> {
        if input.len() == 0 {
            return ParseResult::Bad(input);
        }

        let line_pos = input.iter().take_while(|b| **b != b'\n').count();

        if line_pos == input.len() {
            ParseResult::Good(input, &input[input.len()..])
        } else {
            ParseResult::Good(&input[..line_pos], &input[line_pos + 1..])
        }
    }
}

pub fn line<'i>() -> impl Parser<'i, &'i [u8]> {
    return Line;
}

pub fn point<'i, T: Copy + Default + 'i, P: Parser<'i, T>>(p: P) -> impl Parser<'i, Point<T>> {
    p.skip(expect_byte(b','))
        .repeat_arr::<2>()
        .map(|[a, b]| Point(a, b))
}

pub fn vertex<'i, T: Copy + Default + 'i, P: Parser<'i, T>>(p: P) -> impl Parser<'i, Vertex<T>> {
    p.skip(expect_byte(b','))
        .repeat_arr::<3>()
        .map(|[a, b, c]| Vertex(a, b, c))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expect_byte() {
        assert_eq!(expect_byte(b'H').parse(b"Hello, World"), ParseResult::Good(b'H', b"ello, World"));
    }

    #[test]
    fn test_expect_bytes() {
        assert_eq!(expect_bytes(b"Hello").parse(b"Hello, World"), ParseResult::Good(b"Hello".as_slice(), b", World"));
        assert_eq!(expect_bytes(b"Hello").parse(b"Hello, Earthlings"), ParseResult::Good(b"Hello".as_slice(), b", Earthlings"));
        assert_eq!(expect_bytes(b"Hello").parse(b"Greetings, Earth"), ParseResult::Bad(b"Greetings, Earth"));
    }

    #[test]
    fn test_expect_either_byte() {
        assert_eq!(expect_either_bytes(b"HEY").parse(b"Hello, World"), ParseResult::Good(b'H', b"ello, World"));
        assert_eq!(expect_either_bytes(b"DEF").parse(b"Hello, Earthlings"), ParseResult::Bad(b"Hello, Earthlings"));
        assert_eq!(expect_either_bytes(b"GHI").parse(b"Greetings, Earth"), ParseResult::Good(b'G', b"reetings, Earth"));
    }

    #[test]
    fn test_any_byte() {
        assert_eq!(any_byte().parse(b"Hello, World"), ParseResult::Good(b'H', b"ello, World"));
        assert_eq!(any_byte().parse(b"ello, World"), ParseResult::Good(b'e', b"llo, World"));
        assert_eq!(any_byte().parse(b"llo, World"), ParseResult::Good(b'l', b"lo, World"));
        assert_eq!(any_byte().parse(b"lo, World"), ParseResult::Good(b'l', b"o, World"));
        assert_eq!(any_byte().parse(b""), ParseResult::Bad(b""));
    }

    #[test]
    fn test_or() {
        let parser = expect_byte(b'A').or(expect_byte(b'B')).or(expect_byte(b'C'));

        assert_eq!(parser.parse(b"ABCD"), ParseResult::Good(b'A', b"BCD"));
        assert_eq!(parser.parse(b"BCD"), ParseResult::Good(b'B', b"CD"));
        assert_eq!(parser.parse(b"CD"), ParseResult::Good(b'C', b"D"));
        assert_eq!(parser.parse(b"D"), ParseResult::Bad(b"D"));
    }

    #[test]
    fn test_and() {
        let parser = expect_byte(b'A').and(expect_byte(b'B')).and(expect_byte(b'C'));
        assert_eq!(parser.parse(b"ABCD"), ParseResult::Good(((b'A', b'B'), b'C'), b"D"));
        assert_eq!(parser.parse(b"ABDC"), ParseResult::Bad(b"ABDC"));
        assert_eq!(parser.parse(b"ADBC"), ParseResult::Bad(b"ADBC"));
        assert_eq!(parser.parse(b"DBAC"), ParseResult::Bad(b"DBAC"));
    }

    #[test]
    fn test_and_skip() {
        let parser = expect_byte(b'A').and_discard(expect_byte(b'B')).and_discard(expect_byte(b'C'));
        assert_eq!(parser.parse(b"ABCD"), ParseResult::Good(b'A', b"D"));
        assert_eq!(parser.parse(b"ABDC"), ParseResult::Bad(b"ABDC"));
        assert_eq!(parser.parse(b"ADBC"), ParseResult::Bad(b"ADBC"));
        assert_eq!(parser.parse(b"DBAC"), ParseResult::Bad(b"DBAC"));
    }

    #[test]
    fn test_and_instead() {
        let parser = expect_byte(b'A').and_instead(expect_byte(b'B')).and_instead(expect_byte(b'C'));
        assert_eq!(parser.parse(b"ABCD"), ParseResult::Good(b'C', b"D"));
        assert_eq!(parser.parse(b"ABDC"), ParseResult::Bad(b"ABDC"));
        assert_eq!(parser.parse(b"ADBC"), ParseResult::Bad(b"ADBC"));
        assert_eq!(parser.parse(b"DBAC"), ParseResult::Bad(b"DBAC"));
    }

    #[test]
    fn test_or_repeat_n() {
        let parser = expect_byte(b'A').or(expect_byte(b'B')).or(expect_byte(b'C')).repeat_n(7);

        assert_eq!(parser.parse(b"ABCCCCABBADABBA"), ParseResult::Good(b"ABCCCCA".to_vec(), b"BBADABBA"));
        assert_eq!(parser.parse(b"ABCCDCABBADABBA"), ParseResult::Bad(b"ABCCDCABBADABBA"));
    }

    #[test]
    fn test_or_repeat() {
        let parser = expect_byte(b'A').or(expect_byte(b'B')).or(expect_byte(b'C')).repeat();

        assert_eq!(parser.parse(b"ABCCCCABBADABBA"), ParseResult::Good(b"ABCCCCABBA".to_vec(), b"DABBA"));
        assert_eq!(parser.parse(b"ABCCCCABBAABCCAAADDDFFG"), ParseResult::Good(b"ABCCCCABBAABCCAAA".to_vec(), b"DDDFFG"));
        assert_eq!(parser.parse(b"DBKRKBRKE"), ParseResult::Bad(b"DBKRKBRKE"));
    }

    #[test]
    fn test_or_repeat_until() {
        let parser = expect_byte(b'A').or(expect_byte(b'B')).or(expect_byte(b'C')).repeat_until(expect_byte(b'D'));

        assert_eq!(parser.parse(b"ABCCCCABBADABBA"), ParseResult::Good(b"ABCCCCABBA".to_vec(), b"ABBA"));
        assert_eq!(parser.parse(b"ABCCCCABBAABCCAAADDDFFG"), ParseResult::Good(b"ABCCCCABBAABCCAAA".to_vec(), b"DDFFG"));
        assert_eq!(parser.parse(b"DBKRKBRKE"), ParseResult::Bad(b"DBKRKBRKE"));
        assert_eq!(parser.parse(b"BKRKBRKE"), ParseResult::Bad(b"BKRKBRKE"));
        assert_eq!(parser.parse(b"KRKBRKE"), ParseResult::Bad(b"KRKBRKE"));
    }

    #[test]
    fn test_unsigned_int() {
        assert_eq!(unsigned_int::<u8>().parse(b"112, 493"), ParseResult::Good(112u8, b", 493"));
        assert_eq!(unsigned_int::<u16>().parse(b"11243"), ParseResult::Good(11243u16, b""));
        assert_eq!(unsigned_int::<u32>().parse(b"9954821; 9348211"), ParseResult::Good(9954821u32, b"; 9348211"));
        assert_eq!(unsigned_int::<u64>().parse(b"1238482482348u64"), ParseResult::Good(1238482482348u64, b"u64"));
        assert_eq!(unsigned_int::<u64>().parse(b"three"), ParseResult::Bad(b"three"));
        assert_eq!(unsigned_int::<u128>().parse(b"1238482332432413411148234448"), ParseResult::Good(1238482332432413411148234448u128, b""));
    }

    #[test]
    fn test_signed_int() {
        assert_eq!(signed_int::<i16>().parse(b"112, 493"), ParseResult::Good(112i16, b", 493"));
        assert_eq!(signed_int::<i16>().parse(b"-112, 493"), ParseResult::Good(-112i16, b", 493"));
        assert_eq!(signed_int::<i16>().parse(b"1-12, 493"), ParseResult::Good(1i16, b"-12, 493"));
        assert_eq!(signed_int::<i16>().parse(b"11243"), ParseResult::Good(11243i16, b""));
        assert_eq!(signed_int::<i64>().parse(b"negative three"), ParseResult::Bad(b"negative three"));
    }

    #[test]
    fn test_line() {
        assert_eq!(
            line().parse(b"stuff\nand\nthings"),
            ParseResult::Good(b"stuff".as_slice(), b"and\nthings"),
        );
        assert_eq!(
            line().parse(b"and\nthings"),
            ParseResult::Good(b"and".as_slice(), b"things"),
        );
        assert_eq!(
            line().parse(b"things"),
            ParseResult::Good(b"things".as_slice(), b""),
        );
        assert_eq!(line().parse(b""), ParseResult::Bad(b""));
        assert_eq!(line().parse(b"\n"), ParseResult::Good(b"".as_slice(), b""));
    }

    #[test]
    fn test_map() {
        #[derive(Debug, Eq, PartialEq)]
        enum TestStuff {
            A(u32),
            B(u64),
            C(u16),
        }

        let parser = expect_bytes(b"A:").and_instead(unsigned_int::<u32>()).map(TestStuff::A)
            .or(
                expect_bytes(b"B:").and_instead(unsigned_int::<u64>()).map(TestStuff::B),
            )
            .or(
                expect_bytes(b"C:").and_instead(unsigned_int::<u16>()).map(TestStuff::C),
            );

        assert_eq!(parser.parse(b"A:123"), ParseResult::Good(TestStuff::A(123), b""));
        assert_eq!(parser.parse(b"B:443"), ParseResult::Good(TestStuff::B(443), b""));
        assert_eq!(parser.parse(b"C:123"), ParseResult::Good(TestStuff::C(123), b""));
        assert_eq!(parser.parse(b"D:443"), ParseResult::Bad(b"D:443"));
    }

    #[test]
    fn test_filter() {
        #[derive(Debug, Eq, PartialEq)]
        enum TestStuff {
            A(u32),
            B(u32),
            C(u32),
        }

        let parser = unsigned_int::<u32>().filter(|v| *v > 0 && *v < 10).map(TestStuff::A)
                .or(unsigned_int::<u32>().filter(|v| *v > 10 && *v < 20).map(TestStuff::B))
                .or(unsigned_int::<u32>().filter(|v| *v > 20 && *v < 30).map(TestStuff::C));

        assert_eq!(parser.parse(b"3"), ParseResult::Good(TestStuff::A(3), b""));
        assert_eq!(parser.parse(b"16"), ParseResult::Good(TestStuff::B(16), b""));
        assert_eq!(parser.parse(b"22"), ParseResult::Good(TestStuff::C(22), b""));
        assert_eq!(parser.parse(b"27,18"), ParseResult::Good(TestStuff::C(27), b",18"));
        assert_eq!(parser.parse(b"37"), ParseResult::Bad(b"37"));
    }

    #[test]
    fn test_point() {
        assert_eq!(vertex(unsigned_int::<u16>()).parse(b"14,32,19"), ParseResult::Good(Vertex(14u16, 32u16, 19u16), b""));
        assert_eq!(vertex(unsigned_int::<u32>()).parse(b"93828,1,823944"), ParseResult::Good(Vertex(93828u32, 1u32, 823944u32), b""));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"-1,15,-192"), ParseResult::Good(Vertex(-1i32, 15i32, -192i32), b""));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"-1,15,-192,"), ParseResult::Good(Vertex(-1i32, 15i32, -192i32), b""));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"two,nine,eight"), ParseResult::Bad(b"two,nine,eight"));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"2,nine,eight"), ParseResult::Bad(b"2,nine,eight"));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"2,9,eight"), ParseResult::Bad(b"2,9,eight"));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"<19,39,23>"), ParseResult::Bad(b"<19,39,23>"));
    }

    #[test]
    fn test_quoted_by() {
        assert_eq!(
            vertex(signed_int::<i32>()).quoted_by(b'<', b'>').parse(b"<19,39,23>blurgh"),
            ParseResult::Good(Vertex(19i32, 39, 23), b"blurgh")
        );
        assert_eq!(
            vertex(signed_int::<i32>()).quoted_by(b'<', b'>').parse(b">19,39,23<blurgh"),
            ParseResult::Bad(b">19,39,23<blurgh")
        );
        assert_eq!(
            vertex(signed_int::<i32>()).quoted_by(b'<', b'>').parse(b"<f19,39,23>blurgh"),
            ParseResult::Bad(b"<f19,39,23>blurgh")
        );
    }
}
