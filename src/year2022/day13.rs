use std::cmp::Ordering;
use smallvec::{SmallVec, smallvec};
use common::aoc::Day;
use common::parse3::{choice, Parser, unsigned_int};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Packets", input.len());
    day.note("Pairs", input.array_chunks::<2>().count());
    day.note("Packet Terms", input.iter().map(|p| p.terms.len()).sum::<usize>());

    day.run(1, "", 100000, || part1(&input));
    //day.run(2, "", 10000, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<Packet> {
    Packet::parser()
        .skip_every(b'\n')
        .repeat()
        .parse(data).unwrap()
}

fn part1(input: &[Packet]) -> usize {
    input.array_chunks::<2>()
        .enumerate()
        .filter(|(_, [a, b])| b > a)
        .map(|(i, _)| i + 1)
        .sum()
}

fn part2(input: &[u8]) -> u32 {
    0
}

#[derive(Eq, PartialEq, Debug)]
struct Packet {
    terms: SmallVec<[PacketTerm; 64]>,
}

impl PartialOrd<Self> for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let (ordering, ..) = self.compare_list(other, 0, 0);
        Some(ordering)
    }
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        #[cfg(test)] println!("Comparing \n\t{:?} \n\t{:?}", self, other);

        let (ordering, ..) = self.compare_list(other, 0, 0);
        ordering
    }
}

impl Packet {
    fn compare_list(&self, other: &Packet, st_index: usize, ot_index: usize) -> (Ordering, usize, usize) {
        #[cfg(test)] println!("{} {}", st_index, ot_index);

        let st = self.terms[st_index];
        let ot = other.terms[ot_index];

        match st {
            PacketTerm::List(sp, sl) => {
                let (mut sp, sl) = (sp as usize, sl as usize);
                let (mut op, ol) = match ot {
                    PacketTerm::List(op, ol) => (op as usize, ol as usize),
                    PacketTerm::Number(_) => (ot_index, ot_index + 1),
                };

                loop {
                    #[cfg(test)] println!("  s: {} {}, o: {} {}", sp, sl, op, ol);

                    // Stop if one of the lists have run out.
                    if sp >= sl && op == ol {
                        return (Ordering::Equal, sp, op);
                    } else if sp >= sl && op < ol {
                        return (Ordering::Less, sp + 1, op);
                    } else if sp < sl && op == ol {
                        return (Ordering::Greater, sp, op + 1);
                    }

                    // Continue through the list until a difference emerges
                    match self.compare_list(other, sp, op) {
                        (Ordering::Equal, next_sp, next_op) => {
                            sp = next_sp;
                            op = next_op;
                        }
                        o => { return o; }
                    }
                }
            }
            PacketTerm::Number(sv) => {
                match ot {
                    PacketTerm::Number(ov) => {
                        (sv.cmp(&ov), st_index + 1, ot_index + 1)
                    }
                    PacketTerm::List(..) => {
                        let (ordering, next_si, next_oi) = other.compare_list(self, ot_index, st_index);
                        (ordering.reverse(), next_si, next_oi)
                    }
                }
            }
        }
    }

    fn parser<'i>() -> impl Parser<'i, Packet> {
        #[derive(Copy, Clone)]
        enum Token {
            ListStart,
            ListEnd,
            Number(u16),
        }

        choice((
            b'['.skip(b' ').map_to_value(Token::ListStart),
            b']'.skip(b',').skip(b' ').map_to_value(Token::ListEnd),
            unsigned_int().skip(b',').skip(b' ').map(Token::Number),
        )).repeat_fold_mut(
            || {
                let mut terms: SmallVec<[PacketTerm; 64]> = SmallVec::new();
                let mut stack: SmallVec<[usize; 16]> = SmallVec::new();

                (terms, stack)
            },
            |(terms, stack): &mut (SmallVec<[PacketTerm; 64]>, SmallVec<[usize; 16]>), token: Token| {
                let index = terms.len() as u8;

                match token {
                    Token::ListStart => {
                        terms.push(PacketTerm::List(index + 1, 0));
                        stack.push(terms.len() - 1);
                    }
                    Token::ListEnd => {
                        let list_end_index = stack.pop().unwrap();
                        if let PacketTerm::List(_, end) = &mut terms[list_end_index] {
                            *end = index;
                        }
                    }
                    Token::Number(v) => {
                        terms.push(PacketTerm::Number(v));
                    }
                }
            },
        ).map(|(terms, _)| Packet { terms })
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum PacketTerm {
    Number(u16),
    List(u8, u8),
}

#[cfg(test)]
mod tests {
    use common::parse3::ParseResult;
    use super::*;

    fn p(s: &[u8]) -> Packet { Packet::parser().parse(s).unwrap() }

    const P1_EXAMPLE: &[u8] = b"[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]
";

    #[test]
    fn part1_works_on_example() {
        assert_eq!(part1(&parse(P1_EXAMPLE)), 13);
    }

    #[test]
    fn cmp_works_on_examples() {
        assert_eq!(p(b"[]").cmp(&p(b"[]")), Ordering::Equal);
        assert_eq!(p(b"[3,9,6,3,4]").cmp(&p(b"[3,9,6,3]")), Ordering::Greater);
        assert_eq!(p(b"[1,1,3,1,1]").cmp(&p(b"[1,1,5,1,1]")), Ordering::Less);
        assert_eq!(p(b"[1,1,5,1,1]").cmp(&p(b"[1,1,3,1,1]")), Ordering::Greater);
        assert_eq!(p(b"[[1],[2,3,4]]").cmp(&p(b"[[1],4]")), Ordering::Less);
        assert_eq!(p(b"[[1],[2,3,4]]").cmp(&p(b"[[1],2]")), Ordering::Greater);
        assert_eq!(p(b"[[1],[2,3,4]]").cmp(&p(b"[[1],[2,3]]")), Ordering::Greater);
        assert_eq!(p(b"[[1],4]").cmp(&p(b"[[1],[2,3,4]]")), Ordering::Greater);
        assert_eq!(p(b"[9]").cmp(&p(b"[[8,7,6]]")), Ordering::Greater);
        assert_eq!(p(b"[[4,4],4,4]").cmp(&p(b"[[4,4],4,4,4]")), Ordering::Less);
        assert_eq!(p(b"[7,7,7,7]").cmp(&p(b"[7,7,7]")), Ordering::Greater);
        assert_eq!(p(b"[]").cmp(&p(b"[3]")), Ordering::Less);
        assert_eq!(p(b"[[[]]]").cmp(&p(b"[[]]")), Ordering::Greater);
        assert_eq!(p(b"[[]]").cmp(&p(b"[[]]")), Ordering::Equal);
        assert_eq!(p(b"[[]]").cmp(&p(b"[[[]]]")), Ordering::Less);
        assert_eq!(p(b"[1,[2,[3,[4,[5,6,7]]]],8,9]").cmp(&p(b"[1,[2,[3,[4,[5,6,0]]]],8,9]")), Ordering::Greater);
        assert_eq!(p(b"[[[8,6,4],[6,6,[4]],[10,4,5,[0,3,3,6],[2,9,4]],[0,[4,5,9,1,4]]],[8,1]]").cmp(
            &p(b"[[6,[10,0,9],[[1,3,7,1],[],[],[9,8,2,5]],[2,[3,3,0,0],[10],6,1]],[7,[8,[9,3,8],9],[[9,7,1,2,4],[6,7,9,3,0],4,[7,9,0],10],1],[[1,0,[3,6,7,9,6],[0,3,10,4,6],[8,1,10,2]],[],[3,[3,5,5],[]],[[1,1],7,2]],[[9,[],[]]],[[[0,5,9],[4,3],[10,10,3],2],[1,[0,10]],9]]")
        ), Ordering::Greater);
        assert_eq!(p(b"[1,[2,[3,[4,[5,6,0]]]],8,9]").cmp(&p(b"[1,[2,[3,[4,[5,6,7]]]],8,9]")), Ordering::Less);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5], [1,0,3,3,1]]]]").cmp(
            &p(b"[[[6,5,[4]]], [5,10,5,3],[6],[10]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[3,1,4,[[6]],[[4,6],1,[4]]],[0],[3,[],7]]").cmp(
            &p(b"[[7]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,1]]]]").cmp(
            &p(b"[[[6,5,[4]]],[5,10,5,3],[6],[10]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,1]]]]").cmp(
            &p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,2]]]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[2]]]]]]]").cmp(
            &p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[1]]]]]]]")
        ), Ordering::Greater);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[2]]]]]]]").cmp(
            &p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[[3]]]]]]]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[[3]]]]]]]]").cmp(
            &p(b"[[[3],[2,[],[1,10,5,5],[1,0,3,3,[[[2]]]]]]]")
        ), Ordering::Greater);
        assert_eq!(p(b"[1]").cmp(&p(b"[[1],1]")), Ordering::Less);
        assert_eq!(p(b"[[1],1]").cmp(&p(b"[1]")), Ordering::Greater);
        assert_eq!(p(b"[[0,0],2]").cmp(&p(b"[[0,0],1]")), Ordering::Greater);
        assert_eq!(p(b"[[0,0],1]").cmp(&p(b"[[0,0],2]")), Ordering::Less);
        assert_eq!(p(b"[1]").cmp(&p(b"[[1,2,3]]")), Ordering::Less);
        assert_eq!(p(b"[1,2,3]").cmp(&p(b"[[1]]")), Ordering::Greater);
        assert_eq!(p(b"[[1,2,3]]").cmp(&p(b"[1]")), Ordering::Greater);
        assert_eq!(p(b"[[],7]").cmp(&p(b"[[3]]")), Ordering::Less);
        assert_eq!(p(b"[[1,1],2]").cmp(&p(b"[[1,1],1]")), Ordering::Greater);
        assert_eq!(p(b"[[1,1],1]").cmp(&p(b"[[1,1],2]")), Ordering::Less);
        assert_eq!(p(b"[[0,0,0],4]").cmp(&p(b"[[0,0,0],0]")), Ordering::Greater);
        assert_eq!(p(b"[[],[4,[],0,[],6],[[[5],[4,3,6]],[[7,6],5],[[3,9],2],7],[4,[],2,[3,[],10]]]").cmp(
            &p(b"[[[0,2],[6,[5,9,8],[1,8,1]]],[[[1,3,0],[],0,10],[6],3],[[[1,1,6,2,1]],9],[[[5,0,4],5,[1,2,5,10,2]],[[7,8,4],1]]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[1],[2,3,4]]").cmp(&p(b"[[1],2,3,4]")), Ordering::Greater);
        assert_eq!(p(b"[[1],2,3,4]").cmp(&p(b"[[1],[2,3,4]]")), Ordering::Less);
        assert_eq!(p(b"[[1],4]").cmp(&p(b"[[1],[2,3,4]]")), Ordering::Greater);
        assert_eq!(p(b"[15]").cmp(&p(b"[17]")), Ordering::Less);
        assert_eq!(p(b"[[],[],117]").cmp(&p(b"[[],[],[]]")), Ordering::Greater);
    }

    #[test]
    fn packet_parser_produces_correct_output() {
        assert_eq!(
            Packet::parser().parse(b"[9]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketTerm::List(1, 2), PacketTerm::Number(9),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[[1,2],3,4,5]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketTerm::List(1, 7),
                    PacketTerm::List(2, 4),
                    PacketTerm::Number(1), PacketTerm::Number(2),
                    PacketTerm::Number(3), PacketTerm::Number(4), PacketTerm::Number(5),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[1,[2,[3,[4,[5,6,7]]]],8,9]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketTerm::List(1, 14),
                        PacketTerm::Number(1),
                        PacketTerm::List(3, 12),
                            PacketTerm::Number(2),
                            PacketTerm::List(5, 12),
                                PacketTerm::Number(3),
                                PacketTerm::List(7, 12),
                                    PacketTerm::Number(4),
                                    PacketTerm::List(9, 12),
                                        PacketTerm::Number(5),
                                        PacketTerm::Number(6),
                                        PacketTerm::Number(7),
                    PacketTerm::Number(8),
                    PacketTerm::Number(9),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[[2],1]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketTerm::List(1, 4),
                    PacketTerm::List(2, 3),
                    PacketTerm::Number(2),
                    PacketTerm::Number(1),
                ]
            }, b"")
        );
    }
}