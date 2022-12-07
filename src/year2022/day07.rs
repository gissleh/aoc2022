use common::aoc::{Day, ResultAndCarry};
use common::parse3::{Parser, line, unsigned_int, expect_bytes};

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Total Entries", input.len());
    day.note("Total Files", input.iter().filter(|p| p.is_file()).count());
    day.note("Total Directories", input.iter().filter(|p| p.is_dir()).count());
    day.note("Directories in root", input[0].unwrap_files().len());

    let ResultAndCarry(_, total_size) = day.run(1, "", 10000, || part1(&input));
    day.run(2, "reusing P1 calculations", 10000, || part2(&input, &total_size));
}

enum FSEntry<'i> {
    File(&'i [u8], u32),
    Dir(&'i [u8], Vec<usize>),
}

impl<'i> FSEntry<'i> {
    fn is_dir(&self) -> bool {
        match self {
            FSEntry::Dir(..) => true,
            FSEntry::File(..) => false,
        }
    }

    fn is_file(&self) -> bool {
        match self {
            FSEntry::Dir(..) => false,
            FSEntry::File(..) => true,
        }
    }

    fn unwrap_files(&self) -> &[usize] {
        match self {
            FSEntry::Dir(_, list) => list.as_slice(),
            FSEntry::File(name, _) => panic!("unwrap_files called on file {}", String::from_utf8_lossy(*name).to_string()),
        }
    }
}

fn parse(data: &[u8]) -> Vec<FSEntry> {
    enum InputLine<'i> {
        CD(&'i [u8]),
        CDUp,
        CDSlash,
        LS,
        FileEntry(u32, &'i [u8]),
        DirEntry(&'i [u8]),
    }

    let input_lines = expect_bytes(b"$ ls\n").map(|_| InputLine::LS)
        .or(expect_bytes(b"$ cd /\n").map(|_| InputLine::CDSlash))
        .or(expect_bytes(b"$ cd ..\n").map(|_| InputLine::CDUp))
        .or(expect_bytes(b"$ cd ").and_instead(line()).map(InputLine::CD))
        .or(expect_bytes(b"dir ").and_instead(line()).map(InputLine::DirEntry))
        .or(unsigned_int().and(line()).map(|(s, n)| InputLine::FileEntry(s, n)))
        .repeat()
        .parse(data).unwrap();

    let mut res = Vec::with_capacity(64);
    res.push(FSEntry::Dir(&data[..0], Vec::with_capacity(8)));

    let mut current_stack = Vec::with_capacity(16);
    current_stack.push(0);

    for line in input_lines {
        let current_index = current_stack.last().copied().unwrap_or_default();

        match line {
            InputLine::LS => {}
            InputLine::FileEntry(size, name) => {
                let new_index = res.len();
                res.push(FSEntry::File(name, size));

                if let FSEntry::Dir(_, entries) = &mut res[current_index] {
                    entries.push(new_index);
                }
            }
            InputLine::DirEntry(name) => {
                let new_index = res.len();
                res.push(FSEntry::Dir(name, Vec::with_capacity(8)));

                if let FSEntry::Dir(_, entries) = &mut res[current_index] {
                    entries.push(new_index);
                }
            }
            InputLine::CDSlash => {
                current_stack.clear();
                current_stack.push(0);
            }
            InputLine::CDUp => {
                current_stack.pop();
            }
            InputLine::CD(target) => {
                if let FSEntry::Dir(_, entries) = &res[current_index] {
                    for entry_index in entries {
                        if let FSEntry::Dir(name, _) = &res[*entry_index] {
                            if (*name) == target {
                                current_stack.push(*entry_index);
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    res
}

fn part1(input: &[FSEntry]) -> ResultAndCarry<u32, Vec<u32>> {
    let mut total_size = vec![0u32; input.len()];
    let mut stack = Vec::with_capacity(32);

    stack.push((0usize, false));

    while let Some((index, has_read)) = stack.pop() {
        if has_read {
            total_size[index] += input[index].unwrap_files()
                .iter()
                .map(|v| total_size[*v])
                .sum::<u32>();
        } else {
            stack.push((index, true));

            for sub_index in input[index].unwrap_files().iter() {
                match input[*sub_index] {
                    FSEntry::File(_, size) => { total_size[*sub_index] = size; }
                    FSEntry::Dir(..) => { stack.push((*sub_index, false)); }
                }
            }
        }
    }

    let result = input.iter().enumerate()
        .filter(|(_, v)| v.is_dir())
        .map(|(i, _)| total_size[i])
        .filter(|s| *s <= 100000)
        .sum::<u32>();

    ResultAndCarry(result, total_size)
}

fn part2(input: &[FSEntry], total_size: &[u32]) -> u32 {
    input.iter().enumerate()
        .filter(|(_, v)| v.is_dir())
        .map(|(i, _)| total_size[i])
        .filter(|s| *s >= 30000000 - (70000000 - total_size[0]))
        .min().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &[u8] = b"$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    #[test]
    fn part1_works_on_example() {
        assert_eq!(part1(&parse(EXAMPLE)).0, 95437);
    }

    #[test]
    fn part2_works_on_example() {
        let ResultAndCarry(_, totals) = part1(&parse(EXAMPLE));

        assert_eq!(part2(&parse(EXAMPLE), &totals), 24933642);
    }
}