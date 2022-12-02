use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use num::Integer;

pub trait Parser<T> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T>;

    fn and<T2, P2: Parser<T2> + Sized>(self, p2: P2) -> And<T, T2, Self, P2> where Self: Sized {
        And(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn and_instead<T2, P2: Parser<T2> + Sized>(self, p2: P2) -> AndInstead<T, T2, Self, P2> where Self: Sized {
        AndInstead(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn and_skip<T2, P2: Parser<T2> + Sized>(self, p2: P2) -> AndSkip<T, T2, Self, P2> where Self: Sized {
        AndSkip(self, p2, PhantomData::default(), PhantomData::default())
    }

    fn or<P2: Parser<T> + Sized>(self, p2: P2) -> Or<T, Self, P2> where Self: Sized {
        Or(self, p2, PhantomData::default())
    }

    fn repeat(self) -> Repeat<T, Self> where Self: Sized {
        Repeat(self, PhantomData::default())
    }

    fn repeat_n(self, n: usize) -> RepeatN<T, Self> where Self: Sized {
        RepeatN(n, self, PhantomData::default())
    }

    fn repeat_until<TE, PE: Parser<TE>>(self, ending: PE) -> RepeatUntil<T, TE, Self, PE> where Self: Sized {
        RepeatUntil(self, ending, PhantomData::default(), PhantomData::default())
    }

    fn map<T2, F: Fn(T) -> T2>(self, cb: F) -> Map<T, T2, F, Self> where Self: Sized {
        Map(self, cb, PhantomData::default(), PhantomData::default())
    }
}

pub struct And<T1, T2, P1, P2> (P1, P2, PhantomData<T1>, PhantomData<T2>) where P1: Parser<T1> + Sized, P2: Parser<T2> + Sized;

impl<T1, T2, P1, P2> Parser<(T1, T2)> for And<T1, T2, P1, P2> where P1: Parser<T1> + Sized, P2: Parser<T2> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, (T1, T2)> {
        if let ParseResult::Good(t1, input) = self.0.parse(input) {
            if let ParseResult::Good(t2, input) = self.1.parse(input) {
                return ParseResult::Good((t1, t2), input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct AndSkip<T1, T2, P1, P2> (P1, P2, PhantomData<T1>, PhantomData<T2>) where P1: Parser<T1> + Sized, P2: Parser<T2> + Sized;

impl<T1, T2, P1, P2> Parser<T1> for AndSkip<T1, T2, P1, P2> where P1: Parser<T1> + Sized, P2: Parser<T2> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T1> {
        if let ParseResult::Good(t1, input) = self.0.parse(input) {
            if let ParseResult::Good(_, input) = self.1.parse(input) {
                return ParseResult::Good(t1, input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct AndInstead<T1, T2, P1, P2> (P1, P2, PhantomData<T1>, PhantomData<T2>) where P1: Parser<T1>, P2: Parser<T2>;

impl<T1, T2, P1, P2> Parser<T2> for AndInstead<T1, T2, P1, P2> where P1: Parser<T1>, P2: Parser<T2> {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T2> {
        if let ParseResult::Good(_, input) = self.0.parse(input) {
            if let ParseResult::Good(t2, input) = self.1.parse(input) {
                return ParseResult::Good(t2, input);
            }
        }

        ParseResult::Bad(input)
    }
}

pub struct Map<T1, T2, F, P1> (P1, F, PhantomData<T1>, PhantomData<T2>)
    where P1: Parser<T1> + Sized,
          F: Fn(T1) -> T2;

impl<T1, T2, F, P1> Parser<T2> for Map<T1, T2, F, P1> where P1: Parser<T1> + Sized,
                                                            F: Fn(T1) -> T2 {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T2> {
        match self.0.parse(input) {
            ParseResult::Good(t1, input) => ParseResult::Good(self.1(t1), input),
            ParseResult::Bad(input) => ParseResult::Bad(input),
        }
    }
}

pub struct Or<T, P1, P2> (P1, P2, PhantomData<T>) where P1: Parser<T> + Sized, P2: Parser<T> + Sized;

impl<T, P1, P2> Parser<T> for Or<T, P1, P2> where P1: Parser<T> + Sized, P2: Parser<T> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(t, input) = self.0.parse(input) {
            ParseResult::Good(t, input)
        } else if let ParseResult::Good(t, input) = self.1.parse(input) {
            ParseResult::Good(t, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

pub struct RepeatN<T, P> (usize, P, PhantomData<T>) where P: Parser<T> + Sized;

impl<T, P> Parser<Vec<T>> for RepeatN<T, P> where P: Parser<T> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Vec<T>> {
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

pub struct Repeat<T, P> (P, PhantomData<T>) where P: Parser<T> + Sized;

impl<T, P> Parser<Vec<T>> for Repeat<T, P> where P: Parser<T> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Vec<T>> {
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

pub struct RepeatUntil<TI, TE, PI, PE> (PI, PE, PhantomData<TI>, PhantomData<TE>) where PI: Parser<TI> + Sized, PE: Parser<TE> + Sized;

impl<TI, TE, PI, PE> Parser<Vec<TI>> for RepeatUntil<TI, TE, PI, PE> where PI: Parser<TI> + Sized, PE: Parser<TE> + Sized {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, Vec<TI>> {
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

impl Parser<u8> for ExpectByte {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        match input.first() {
            Some(v) if *v == self.0 => ParseResult::Good(*v, &input[1..]),
            _ => ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn expect_byte(byte: u8) -> impl Parser<u8> {
    ExpectByte(byte)
}

struct AnyByte;

impl Parser<u8> for AnyByte {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        match input.first() {
            Some(v) => ParseResult::Good(*v, &input[1..]),
            _ => ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn any_byte() -> impl Parser<u8> {
    AnyByte
}

struct ExpectBytes<'p>(&'p [u8]);

impl<'p> Parser<&'p [u8]> for ExpectBytes<'p> {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, &'p [u8]> {
        if self.0.len() == input.len() {
            ParseResult::Bad(input)
        } else if input[..self.0.len()].eq(self.0) {
            ParseResult::Good(self.0, &input[self.0.len()..])
        } else {
            ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn expect_bytes(bytes: &[u8]) -> impl Parser<&[u8]> {
    ExpectBytes(bytes)
}

struct ExpectEitherByte<'p>(&'p [u8]);

impl<'p> Parser<u8> for ExpectEitherByte<'p> {
    #[inline]
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
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
pub fn expect_either_bytes(bytes: &[u8]) -> impl Parser<u8> + '_ {
    ExpectEitherByte(bytes)
}

struct UnsignedInt<T: Integer + Copy + From<u8>> (PhantomData<T>);

#[inline]
pub fn unsigned_int<T: Integer + Copy + From<u8> + Default>() -> impl Parser<T> {
    return UnsignedInt(PhantomData::default());
}

impl<T: Integer + Copy + From<u8>> Parser<T> for UnsignedInt<T> {
    fn parse<'i>(&self, input: &'i [u8]) -> ParseResult<'i, T> {
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
        let parser = expect_byte(b'A').and_skip(expect_byte(b'B')).and_skip(expect_byte(b'C'));
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
}
