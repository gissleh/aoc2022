use std::fmt::{Debug, Display, Formatter};
use std::marker::PhantomData;
use std::ops::{Neg, Range, RangeInclusive};
use num::Integer;
use crate::geo::{Point, Vertex};

pub trait Parser<'i, T> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T>;

    fn first_parsable(&self, input: &'i [u8]) -> ParseResult<'i, (T, usize)> {
        for off in 0..input.len() {
            return match self.parse(&input[off..]) {
                ParseResult::Good(t, new_input) => ParseResult::Good((t, off), new_input),
                ParseResult::Bad(old_input) => ParseResult::Bad(old_input),
            };
        }

        ParseResult::Bad(input)
    }

    fn iterate(self, input: &'i [u8]) -> ParseIterator<'i, Self, T> where Self: Sized {
        ParseIterator {
            input,

            parser: self,
            spooky_ghost: PhantomData::default(),
        }
    }

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

    fn skip_every<T2, P2: Parser<'i, T2> + Sized>(self, p2: P2) -> SkipEvery<T2, Self, P2> where Self: Sized {
        SkipEvery(self, p2, PhantomData::default())
    }

    fn or<P2: Parser<'i, T> + Sized>(self, p2: P2) -> Or<'i, T, Self, P2> where Self: Sized {
        Or(self, p2, PhantomData::default())
    }

    fn repeat(self) -> Repeat<'i, T, Self> where Self: Sized {
        Repeat(self, PhantomData::default())
    }

    fn repeat_fold_mut<TF, IF: Fn() -> TF, SF: Fn(&mut TF, T)>(self, init_fn: IF, step_fn: SF) -> RepeatFoldMut<'i, T, TF, SF, IF, Self> where Self: Sized {
        RepeatFoldMut(self, init_fn, step_fn, PhantomData::default(), PhantomData::default())
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

    fn repeat_delimited<TD, PD: Parser<'i, TD>>(self, delim: PD) -> RepeatDelimited<'i, T, TD, Self, PD> where Self: Sized {
        RepeatDelimited(self, delim, PhantomData::default(), PhantomData::default())
    }

    fn quoted_by<TP, TS, PP: Parser<'i, TP>, PS: Parser<'i, TS>>(self, prefix: PP, suffix: PS) -> Quoted<'i, T, TP, TS, Self, PP, PS> where Self: Sized {
        Quoted(self, prefix, suffix, PhantomData::default(), PhantomData::default(), PhantomData::default())
    }

    fn map<T2, F: Fn(T) -> T2>(self, cb: F) -> Map<'i, T, T2, F, Self> where Self: Sized {
        Map(self, cb, PhantomData::default(), PhantomData::default())
    }

    fn map_to_value<T2: Copy>(self, t2: T2) -> MapValue<'i, T, T2, Self> where Self: Sized {
        MapValue(self, t2, PhantomData::default(), PhantomData::default())
    }

    fn filter<F: Fn(&T) -> bool>(self, cb: F) -> Filter<'i, T, F, Self> where Self: Sized {
        Filter(self, cb, PhantomData::default())
    }
}

pub struct ParseIterator<'i, P, T> {
    parser: P,
    input: &'i [u8],
    spooky_ghost: PhantomData<T>,
}

impl<'i, P, T> Iterator for ParseIterator<'i, P, T> where P: Parser<'i, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parser.parse(self.input) {
            ParseResult::Good(t, new_input) => {
                self.input = new_input;
                Some(t)
            }
            ParseResult::Bad(_) => None
        }
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

pub struct SkipEvery<T2, P1, P2> (P1, P2, PhantomData<T2>);

impl<'i, T1, T2, P1: Parser<'i, T1>, P2: Parser<'i, T2>> Parser<'i, T1> for SkipEvery<T2, P1, P2> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T1> {
        if let ParseResult::Good(t1, mut input) = self.0.parse(input) {
            while let ParseResult::Good(_, new_input) = self.1.parse(input) {
                input = new_input;
            }

            ParseResult::Good(t1, input)
        } else {
            ParseResult::Bad(input)
        }
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

pub struct MapValue<'i, T1, T2, P1> (P1, T2, PhantomData<&'i T1>, PhantomData<T2>)
    where P1: Parser<'i, T1> + Sized,
          T2: Copy;

impl<'i, T1, T2, P1> Parser<'i, T2> for MapValue<'i, T1, T2, P1>
    where P1: Parser<'i, T1> + Sized,
          T2: Copy {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T2> {
        match self.0.parse(input) {
            ParseResult::Good(_, input) => ParseResult::Good(self.1, input),
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

pub struct RepeatFoldMut<'i, T, TF, SF, IF, P> (P, IF, SF, PhantomData<&'i T>, PhantomData<TF>)
    where P: Parser<'i, T> + Sized,
          IF: Fn() -> TF,
          SF: Fn(&mut TF, T);

impl<'i, T, TF, SF, IF, P> Parser<'i, TF> for RepeatFoldMut<'i, T, TF, SF, IF, P>
    where P: Parser<'i, T> + Sized,
          IF: Fn() -> TF,
          SF: Fn(&mut TF, T) {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, TF> {
        let mut fold_value = self.1();
        let mut current_input = input;
        let mut first = true;

        loop {
            match self.0.parse(current_input) {
                ParseResult::Good(t, new_input) => {
                    self.2(&mut fold_value, t);
                    current_input = new_input;
                    first = false;
                }
                ParseResult::Bad(_) => {
                    if first {
                        return ParseResult::Bad(input);
                    }

                    break;
                }
            }
        }

        ParseResult::Good(fold_value, current_input)
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

pub struct Quoted<'i, T, TP, TS, P, PP, PS> (P, PP, PS, PhantomData<&'i T>, PhantomData<TP>, PhantomData<TS>);

impl<'i, T, TP, TS, P, PP, PS> Parser<'i, T> for Quoted<'i, T, TP, TS, P, PP, PS>
    where P: Parser<'i, T> + Sized,
          PP: Parser<'i, TP> + Sized,
          PS: Parser<'i, TS> + Sized
{
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(_, input) = self.1.parse(input) {
            if let ParseResult::Good((_, offset), input_after_delim) = self.2.first_parsable(&input) {
                if let ParseResult::Good(t, new_input) = self.0.parse(&input[..offset]) {
                    return if new_input.len() == 0 {
                        ParseResult::Good(t, input_after_delim)
                    } else {
                        ParseResult::Bad(input)
                    };
                }
            }
        }

        ParseResult::Bad(input)
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

pub struct RepeatDelimited<'i, TI, TD, PI, PD> (PI, PD, PhantomData<&'i TI>, PhantomData<TD>) where PI: Parser<'i, TI> + Sized, PD: Parser<'i, TD> + Sized;

impl<'i, TI, TD, PI, PD> Parser<'i, Vec<TI>> for RepeatDelimited<'i, TI, TD, PI, PD> where PI: Parser<'i, TI> + Sized, PD: Parser<'i, TD> + Sized {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, Vec<TI>> {
        let mut current_input = input;
        let mut res = Vec::with_capacity(64);

        loop {
            match self.0.parse(current_input) {
                ParseResult::Good(t, input_after_term) => {
                    res.push(t);
                    current_input = input_after_term;

                    if let ParseResult::Good(_, input_after_delim) = self.1.parse(current_input) {
                        current_input = input_after_delim;
                    } else {
                        return ParseResult::Good(res, input_after_term);
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

    #[inline]
    pub fn unwrap_and_input(self) -> (T, &'i [u8]) {
        match self {
            ParseResult::Good(v, i) => (v, i),
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

#[derive(Copy, Clone)]
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

impl<'i> Parser<'i, u8> for u8 {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, u8> {
        match input.first() {
            Some(v) if *v == *self => ParseResult::Good(*self, &input[1..]),
            _ => ParseResult::Bad(input)
        }
    }

    #[inline]
    fn first_parsable(&self, input: &'i [u8]) -> ParseResult<'i, (u8, usize)> {
        match input.iter().enumerate().find(|(_, v)| *v == self) {
            Some((pos, v)) => ParseResult::Good((*v, pos), &input[pos + 1..]),
            None => ParseResult::Bad(input)
        }
    }
}

#[inline]
pub fn expect_byte<'i>(byte: u8) -> impl Parser<'i, u8> + Copy { byte }

#[derive(Copy, Clone)]
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
pub const fn any_byte<'i>() -> impl Parser<'i, u8> + Copy {
    AnyByte
}

impl<'i> Parser<'i, &'i [u8]> for &'static [u8] {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'i [u8]> {
        if self.len() >= input.len() {
            ParseResult::Bad(input)
        } else if (input[..self.len()]).eq(*self) {
            ParseResult::Good(&input[..self.len()], &input[self.len()..])
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, const N: usize> Parser<'i, &'i [u8]> for &[u8; N] {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'i [u8]> {
        if N >= input.len() {
            ParseResult::Bad(input)
        } else if (input.split_array_ref::<N>()).0.eq(*self) {
            ParseResult::Good(&input[..N], &input[N..])
        } else {
            ParseResult::Bad(input)
        }
    }

    #[inline]
    fn first_parsable(&self, input: &'i [u8]) -> ParseResult<'i, (&'i [u8], usize)> {
        match input.windows(self.len()).position(|sub| sub == self.as_slice()) {
            Some(pos) => {
                let end = pos + self.len();
                ParseResult::Good((&input[pos..end], pos), &input[end..])
            }
            None => ParseResult::Bad(input)
        }
    }
}

impl<'i, T> Parser<'i, T> for Range<T>
    where T: Integer + Copy + From<u8> + Default + Ord {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, new_input) = unsigned_int::<T>().parse(input) {
            if v >= self.start && v < self.end {
                ParseResult::Good(v, new_input)
            } else {
                ParseResult::Bad(input)
            }
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T> Parser<'i, T> for RangeInclusive<T>
    where T: Integer + Copy + From<u8> + Default + Ord {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, new_input) = unsigned_int::<T>().parse(input) {
            if v >= *self.start() && v <= *self.end() {
                ParseResult::Good(v, new_input)
            } else {
                ParseResult::Bad(input)
            }
        } else {
            ParseResult::Bad(input)
        }
    }
}

#[inline]
pub const fn expect_bytes<'i>(bytes: &'static [u8]) -> impl Parser<'i, &'i [u8]> + Copy {
    bytes
}

#[derive(Copy, Clone)]
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
pub const fn expect_either_bytes<'i, 'p>(bytes: &'p [u8]) -> impl Parser<'i, u8> + Copy + '_ {
    ExpectEitherByte(bytes)
}

#[derive(Copy, Clone)]
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
pub fn unsigned_int<'i, T: Integer + Copy + From<u8> + Default>() -> impl Parser<'i, T> + Copy {
    return UnsignedInt(PhantomData::default());
}

#[derive(Copy, Clone)]
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
pub fn signed_int<'i, T: Integer + Copy + From<u8> + Neg<Output=T>>() -> impl Parser<'i, T> + Copy {
    return SignedInt(PhantomData::default());
}

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
struct Everything;

impl<'i> Parser<'i, &'i [u8]> for Everything {
    #[inline]
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, &'i [u8]> {
        return if input.len() > 0 {
            ParseResult::Good(input, &input[input.len()..])
        } else {
            ParseResult::Bad(input)
        };
    }
}

pub fn everything<'i>() -> impl Parser<'i, &'i [u8]> {
    return Everything;
}

pub trait Choices<'i, T> {
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T>;
}

impl<'i, const N: usize, T, P: Parser<'i, T>> Choices<'i, T> for [P; N] {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        for i in 0..N {
            if let ParseResult::Good(t, input) = self[i].parse(input) {
                return ParseResult::Good(t, input);
            }
        }

        ParseResult::Bad(input)
    }
}

// Things to learn: Code generation

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>> Choices<'i, T> for (P1, P2) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>, P4: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3, P4) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.3.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>, P4: Parser<'i, T>, P5: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3, P4, P5) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.3.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.4.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>, P4: Parser<'i, T>, P5: Parser<'i, T>, P6: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3, P4, P5, P6) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.3.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.4.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.5.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>, P4: Parser<'i, T>, P5: Parser<'i, T>, P6: Parser<'i, T>, P7: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3, P4, P5, P6, P7) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.3.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.4.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.5.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.6.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

impl<'i, T, P1: Parser<'i, T>, P2: Parser<'i, T>, P3: Parser<'i, T>, P4: Parser<'i, T>, P5: Parser<'i, T>, P6: Parser<'i, T>, P7: Parser<'i, T>, P8: Parser<'i, T>> Choices<'i, T> for (P1, P2, P3, P4, P5, P6, P7, P8) {
    #[inline]
    fn parse_choice(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        if let ParseResult::Good(v, input) = self.0.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.1.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.2.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.3.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.4.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.5.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.6.parse(input) {
            ParseResult::Good(v, input)
        } else if let ParseResult::Good(v, input) = self.7.parse(input) {
            ParseResult::Good(v, input)
        } else {
            ParseResult::Bad(input)
        }
    }
}

struct Choice<C> (C);

impl<'i, C, T> Parser<'i, T> for Choice<C> where C: Choices<'i, T> {
    fn parse(&self, input: &'i [u8]) -> ParseResult<'i, T> {
        self.0.parse_choice(input)
    }
}

pub fn choice<'i, T, C: Choices<'i, T>>(choices: C) -> impl Parser<'i, T> {
    Choice(choices)
}

pub fn point<'i, T: Copy + Default + 'i, P: Parser<'i, T> + Copy>(p: P) -> impl Parser<'i, Point<T>> {
    p.and_discard(expect_byte(b','))
        .and(p)
        .map(|(x, y)| Point(x, y))
}

pub fn vertex<'i, T: Copy + Default + 'i, P: Parser<'i, T> + Copy>(p: P) -> impl Parser<'i, Vertex<T>> {
    p.and_discard(expect_byte(b','))
        .and(p)
        .and_discard(expect_byte(b','))
        .and(p)
        .map(|((x, y), z)| Vertex(x, y, z))
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
        let parser = b'A'.and_discard(b'B').and_discard(b'C');
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
        assert_eq!((0..16u32).parse(b"9954821; 9348211"), ParseResult::Bad(b"9954821; 9348211"));
        assert_eq!((0..9954822u32).parse(b"9954821; 9348211"), ParseResult::Good(9954821u32, b"; 9348211"));
        assert_eq!((0..=9954821u32).parse(b"9954821; 9348211"), ParseResult::Good(9954821u32, b"; 9348211"));
        assert_eq!((0..=9954820u32).parse(b"9954821; 9348211"), ParseResult::Bad(b"9954821; 9348211"));
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
    fn test_repeat_delimited() {
        let parser = signed_int::<i32>().repeat_delimited(b", ");

        assert_eq!(parser.parse(b"112, 493"), ParseResult::Good(vec![112i32, 493], b""));
        assert_eq!(parser.parse(b"112, 493: loaf"), ParseResult::Good(vec![112i32, 493], b": loaf"));
        assert_eq!(parser.parse(b"112, 493, loaf"), ParseResult::Bad(b"112, 493, loaf"));
    }

    #[test]
    fn test_skip() {
        let parser = b"test:".skip(b' ').and_instead(unsigned_int::<u32>());

        assert_eq!(parser.parse(b"test: 64"), ParseResult::Good(64u32, b""));
        assert_eq!(parser.parse(b"test:112"), ParseResult::Good(112u32, b""));
        assert_eq!(parser.parse(b"test:  643 "), ParseResult::Bad(b"test:  643 "));

        let parser = b"test:".skip_every(b' ').and_instead(unsigned_int::<u32>());
        assert_eq!(parser.parse(b"test: 64"), ParseResult::Good(64u32, b""));
        assert_eq!(parser.parse(b"test:112"), ParseResult::Good(112u32, b""));
        assert_eq!(parser.parse(b"test:  643 "), ParseResult::Good(643u32, b" "));
        assert_eq!(parser.parse(b"test:        712"), ParseResult::Good(712u32, b""));
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
        assert_eq!(point(unsigned_int::<u16>()).parse(b"14,32,19"), ParseResult::Good(Point(14u16, 32u16), b",19"));
        assert_eq!(vertex(unsigned_int::<u32>()).parse(b"93828,1,823944"), ParseResult::Good(Vertex(93828u32, 1u32, 823944u32), b""));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"-1,15,-192"), ParseResult::Good(Vertex(-1i32, 15i32, -192i32), b""));
        assert_eq!(vertex(signed_int::<i32>()).parse(b"-1,15,-192,"), ParseResult::Good(Vertex(-1i32, 15i32, -192i32), b","));
        assert_eq!(point(signed_int::<i32>()).parse(b"-1,15,-192,"), ParseResult::Good(Point(-1i32, 15i32), b",-192,"));
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
        assert_eq!(
            unsigned_int::<u16>()
                .quoted_by(b"Number(", b')')
                .parse(b"Number(5582), String(\"Hello\")"),
            ParseResult::Good(5582u16, b", String(\"Hello\")")
        );
        assert_eq!(
            unsigned_int::<u16>()
                .repeat_delimited(b",".skip_every(b' '))
                .quoted_by(b'[', b"]")
                .parse(b"[1,    43,    24,15,     112];"),
            ParseResult::Good(vec![1u16, 43, 24, 15, 112], b";")
        );
        assert_eq!(
            everything().quoted_by(b'"', b'"').parse(b"\"Hello, World!\""),
            ParseResult::Good(b"Hello, World!".as_slice(), b"")
        );
        assert_eq!(
            everything().quoted_by(b"say(\"", b"\");").parse(b"say(\"Hello, World\"); i += 1;"),
            ParseResult::Good(b"Hello, World".as_slice(), b" i += 1;")
        );
    }

    #[test]
    fn test_choices_array() {
        let parser = choice([b'a', b'b', b'c']);
        assert_eq!(parser.parse(b"abcd"), ParseResult::Good(b'a', b"bcd"));
        assert_eq!(parser.parse(b"bcd"), ParseResult::Good(b'b', b"cd"));
        assert_eq!(parser.parse(b"cd"), ParseResult::Good(b'c', b"d"));
        assert_eq!(parser.parse(b"d"), ParseResult::Bad(b"d"));

        let parser = choice([
            b'a'.map_to_value(0u32),
            b'b'.map_to_value(1u32),
            b'c'.map_to_value(2u32),
        ]);
        assert_eq!(parser.parse(b"abcd"), ParseResult::Good(0u32, b"bcd"));
        assert_eq!(parser.parse(b"bcd"), ParseResult::Good(1u32, b"cd"));
        assert_eq!(parser.parse(b"cd"), ParseResult::Good(2u32, b"d"));
        assert_eq!(parser.parse(b"d"), ParseResult::Bad(b"d"));

        #[derive(Eq, PartialEq, Copy, Clone, Debug)]
        enum Greeting { Hello, Greetings, Hi }
        let parser = choice([
            b"hello ".as_slice().map_to_value(Greeting::Hello).and(line()),
            b"hello, ".as_slice().map_to_value(Greeting::Hello).and(line()),
            b"greetings ".as_slice().map_to_value(Greeting::Greetings).and(line()),
            b"greetings, ".as_slice().map_to_value(Greeting::Greetings).and(line()),
            b"hi ".as_slice().map_to_value(Greeting::Hi).and(line()),
            b"hi, ".as_slice().map_to_value(Greeting::Hi).and(line()),
        ]);
        assert_eq!(parser.parse(b"hi bob"), ParseResult::Good((Greeting::Hi, b"bob".as_slice()), b""));
        assert_eq!(parser.parse(b"greetings, archebald"), ParseResult::Good((Greeting::Greetings, b"archebald".as_slice()), b""));
    }

    #[test]
    fn test_choices_tuple() {
        #[derive(Debug, Eq, PartialEq)]
        enum Instruction<'i> {
            Add(u16, u16),
            Sub(u16, u16),
            Mul(u16, u16),
            PrintStr(&'i [u8]),
            PrintVal(u16),
            PrintAddrs(Vec<u16>),
        }
        use Instruction::*;
        let number = unsigned_int::<u16>();
        let parser = choice((
            b"add ".and_instead(number)
                .and_discard(b' ')
                .and(number)
                .map(|(a, b)| Instruction::Add(a, b)),
            b"sub ".and_instead(number)
                .and_discard(b' ')
                .and(number)
                .map(|(a, b)| Instruction::Sub(a, b)),
            b"mul ".and_instead(number)
                .and_discard(b' ')
                .and(number)
                .map(|(a, b)| Instruction::Mul(a, b)),
            b"print_str ".and_instead(line()).map(Instruction::PrintStr),
            b"print_val ".and_instead(number).map(Instruction::PrintVal),
            b"print_addrs ".and_instead(
                number.repeat_delimited(b' ')
            ).map(Instruction::PrintAddrs),
        ));

        assert_eq!(parser.parse(b"add 14 9"), ParseResult::Good(Add(14, 9), b""));
        assert_eq!(parser.parse(b"print_str Hello World"), ParseResult::Good(PrintStr(b"Hello World"), b""));
        assert_eq!(parser.parse(b"print_addrs 1 2 3 5 7 9"), ParseResult::Good(PrintAddrs(vec![1u16, 2, 3, 5, 7, 9]), b""));
        assert_eq!(parser.parse(b"eat_food 123"), ParseResult::Bad(b"eat_food 123"));
    }
}
