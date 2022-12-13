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
    day.run(2, "", 10000, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<Packet> {
    Packet::parser()
        .skip_every(b'\n')
        .repeat()
        .parse(data).unwrap()
}

fn part1(packets: &[Packet]) -> usize {
    packets.array_chunks::<2>()
        .enumerate()
        .filter(|(_, [a, b])| a < b)
        .map(|(i, _)| i + 1)
        .sum()
}

fn part2(packets: &[Packet]) -> usize {
    let mut packets = packets.to_vec();
    packets.push(Packet::new_divider(2));
    packets.push(Packet::new_divider(6));
    packets.sort();

    let mut divider_2_pos = 0;
    for (i, packet) in packets.iter().enumerate() {
        if let Some(divider) = packet.get_divider() {
            if divider == 2 {
                divider_2_pos = i + 1;
            } else if divider == 6 {
                return divider_2_pos * (i + 1)
            }
        }
    }

    0
}

#[derive(Eq, PartialEq, Debug, Clone)]
struct Packet {
    terms: SmallVec<[PacketPart; 64]>,
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
            PacketPart::List(sp, sl) => {
                let (mut sp, sl) = (sp as usize, sl as usize);
                let (mut op, ol) = match ot {
                    PacketPart::List(op, ol) => (op as usize, ol as usize),
                    PacketPart::Number(_) => (ot_index, ot_index + 1),
                };

                loop {
                    #[cfg(test)] println!("{} {}:  s: {} {}, o: {} {}", st_index, ot_index, sp, sl, op, ol);

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
            PacketPart::Number(sv) => {
                match ot {
                    PacketPart::Number(ov) => {
                        (sv.cmp(&ov), st_index + 1, ot_index + 1)
                    }
                    PacketPart::List(..) => {
                        let (ordering, next_si, next_oi) = other.compare_list(self, ot_index, st_index);
                        (ordering.reverse(), next_oi, next_si) // They next's need to be swapped here too.
                    }
                }
            }
        }
    }

    fn get_divider(&self) -> Option<u16> {
        if self.terms.len() == 3
            && self.terms[0] == PacketPart::List(1, 3)
            && self.terms[1] == PacketPart::List(2, 3) {
            if let PacketPart::Number(v) = self.terms[2] {
                return Some(v);
            }
        }

        None
    }

    fn new_divider(v: u16) -> Packet {
        Packet {
            terms: smallvec![
                PacketPart::List(1, 3),
                PacketPart::List(2, 3),
                PacketPart::Number(v)
            ],
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
                let terms: SmallVec<[PacketPart; 64]> = SmallVec::new();
                let stack: SmallVec<[usize; 16]> = SmallVec::new();

                (terms, stack)
            },
            |(terms, stack), token| {
                let index = terms.len() as u8;

                match token {
                    Token::ListStart => {
                        terms.push(PacketPart::List(index + 1, 0));
                        stack.push(terms.len() - 1);
                    }
                    Token::ListEnd => {
                        let list_end_index = stack.pop().unwrap();
                        if let PacketPart::List(_, end) = &mut terms[list_end_index] {
                            *end = index;
                        }
                    }
                    Token::Number(v) => {
                        terms.push(PacketPart::Number(v));
                    }
                }
            },
        ).map(|(terms, _)| Packet { terms })
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum PacketPart {
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

    const P1_EXAMPLE_REV: &[u8] = b"[1,1,5,1,1]
[1,1,3,1,1]

[[1],4]
[[1],[2,3,4]]

[[8,7,6]]
[9]

[[4,4],4,4,4]
[[4,4],4,4]

[7,7,7]
[7,7,7,7]

[3]
[]

[[]]
[[[]]]

[1,[2,[3,[4,[5,6,0]]]],8,9]
[1,[2,[3,[4,[5,6,7]]]],8,9]
";

    #[test]
    fn part1_works_on_example() {
        assert_eq!(part1(&parse(P1_EXAMPLE)), 13);
        assert_eq!(part1(&parse(P1_EXAMPLE_REV)), 23);
    }

    #[test]
    fn parse_parses_multiple_packets() {
        let packets = parse(P1_EXAMPLE);

        assert_eq!(packets.len(), 16);
        assert_eq!(packets[0], p(b"[1,1,3,1,1]"));
        assert_eq!(packets[1], p(b"[1,1,5,1,1]"));
        assert_eq!(packets[2], p(b"[[1],[2,3,4]]"));
        assert_eq!(packets[3], p(b"[[1],4]"));
        assert_eq!(packets[4], p(b"[9]"));
        assert_eq!(packets[5], p(b"[[8,7,6]]"));
        assert_eq!(packets[6], p(b"[[4,4],4,4]"));
        assert_eq!(packets[7], p(b"[[4,4],4,4,4]"));
        assert_eq!(packets[8], p(b"[7,7,7,7]"));
        assert_eq!(packets[9], p(b"[7,7,7]"));
        assert_eq!(packets[10], p(b"[]"));
        assert_eq!(packets[11], p(b"[3]"));
        assert_eq!(packets[12], p(b"[[[]]]"));
        assert_eq!(packets[13], p(b"[[]]"));
        assert_eq!(packets[14], p(b"[1,[2,[3,[4,[5,6,7]]]],8,9]"));
        assert_eq!(packets[15], p(b"[1,[2,[3,[4,[5,6,0]]]],8,9]"));
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
        assert_eq!(p(b"[[9]]").cmp(&p(b"[[[9]]]")), Ordering::Equal);
        assert_eq!(p(b"[[9]]").cmp(&p(b"[[[[9]]]]")), Ordering::Equal);
        assert_eq!(p(b"[[[[9]]],[],[9,[8],8],[[7,[],4],8,[[],1,[0,4,7],[3,10,6]]]]").cmp(
            &p(b"[[9],[5,[6,7],1,7],[[[0],9,1,[]]]]")
        ), Ordering::Less);
        assert_eq!(p(b"[[9],[5,[6,7],1,7],[[[0],9,1,[]]]]").cmp(
            &p(b"[[[[9]]],[],[9,[8],8],[[7,[],4],8,[[],1,[0,4,7],[3,10,6]]]]")
        ), Ordering::Greater);
    }

    #[test]
    fn packet_parser_produces_correct_output() {
        assert_eq!(
            Packet::parser().parse(b"[9]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketPart::List(1, 2), PacketPart::Number(9),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[[1,2],3,4,5]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketPart::List(1, 7),
                    PacketPart::List(2, 4),
                    PacketPart::Number(1), PacketPart::Number(2),
                    PacketPart::Number(3), PacketPart::Number(4), PacketPart::Number(5),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[1,[2,[3,[4,[5,6,7]]]],8,9]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketPart::List(1, 14),
                        PacketPart::Number(1),
                        PacketPart::List(3, 12),
                            PacketPart::Number(2),
                            PacketPart::List(5, 12),
                                PacketPart::Number(3),
                                PacketPart::List(7, 12),
                                    PacketPart::Number(4),
                                    PacketPart::List(9, 12),
                                        PacketPart::Number(5),
                                        PacketPart::Number(6),
                                        PacketPart::Number(7),
                    PacketPart::Number(8),
                    PacketPart::Number(9),
                ]
            }, b"")
        );
        assert_eq!(
            Packet::parser().parse(b"[[2],1]"),
            ParseResult::Good(Packet {
                terms: smallvec![
                    PacketPart::List(1, 4),
                    PacketPart::List(2, 3),
                    PacketPart::Number(2),
                    PacketPart::Number(1),
                ]
            }, b"")
        );
    }
}