use arrayvec::ArrayVec;
use common::aoc::Day;

pub fn main(day: &mut Day, input: &[u8]) {
    let input = day.run_parse(1000, || parse(input));

    day.note("Jets", input.len());

    day.run(1, "", 100, || part1(&input));
    day.run(2, "", 100, || part2(&input));
}

fn parse(data: &[u8]) -> Vec<i64> {
    data.iter()
        .filter(|v| **v != b'\n' && **v != b'\r')
        .map(|v| if *v == b'>' { 1 } else { -1 })
        .collect()
}

fn part1(jets: &[i64]) -> i64 {
    let mut top_y = 0;
    let mut tower = Tower::with_capacity(4096);
    let mut jets = jets.iter().copied().cycle();

    for template in ROCK_TEMPLATES.iter().cycle().take(2022) {
        let rock = Rock::simulate_fall(top_y, &mut jets, &tower, &template).unwrap();
        if rock.y < top_y { top_y = rock.y; }

        tower.place_rock(&rock);
    }

    #[cfg(test)] tower.render();

    -top_y
}

fn part2(jets: &[i64]) -> i64 {
    let mut top_y = 0;
    let mut tower = Tower::with_capacity(4096);
    let mut jets = jets.iter().copied().cycle();

    let mut border: Vec<u8> = Vec::new();
    let mut border_indices: ArrayVec<usize, 4> = ArrayVec::new();
    let mut border_ys: ArrayVec<i64, 4> = ArrayVec::new();
    let mut offset = 0usize;
    let mut height_offset = 0i64;

    for (index, template) in ROCK_TEMPLATES.iter().cycle().enumerate() {
        let rock = Rock::simulate_fall(top_y, &mut jets, &tower, &template).unwrap();
        tower.place_rock(&rock);

        if rock.y < top_y {
            top_y = rock.y;

            if top_y < -2000 && border.is_empty() {
                let i = tower.get_y(-1500);
                border = Vec::from(&tower.data[i..i + 8]);
            }

            if top_y < -2500 && !border_indices.is_full() {
                let i = tower.get_y(top_y);
                if &tower.data[i - 128..i - 120] == &border {
                    border_indices.push(index);
                    border_ys.push(top_y);

                    if border_indices.is_full() {
                        let cycle_len = border_indices[1] - border_indices[0];
                        let cycle_height =  border_ys[1] - border_ys[0];

                        let cycle_count = ((999999999999 - index) / cycle_len) - 1;

                        offset = cycle_len * cycle_count;
                        height_offset = cycle_height * (cycle_count as i64);
                    }
                }
            }
        }

        if index + offset == 999999999999 {
            break;
        }
    }

    #[cfg(test)] tower.render();

    -(top_y + height_offset)
}

#[derive(Clone)]
struct Tower {
    data: Vec<u8>,
}

impl Tower {
    fn get_y(&self, y: i64) -> usize {
        -(y + 1) as usize
    }

    #[cfg(test)]
    fn render(&self) {
        let mut first = false;
        for y in (0..self.data.len()).rev() {
            if !first && self.data[y] == 0 {
                continue;
            }
            first = true;

            for x in 0..7 {
                if self.data[y] & (1 << x) != 0 {
                    print!("#");
                } else {
                    print!(".");
                }
            }

            println!();
        }
    }

    fn can_place_rock<'r>(&self, rock: &'r Rock) -> bool {
        let y = self.get_y(rock.y);
        for i in 0..rock.points.len() {
            let y_index = y - i;
            if self.data.len() <= y_index {
                continue;
            }

            let mask = rock.points[i] << rock.x;
            if self.data[y_index] & mask != 0 {
                return false;
            }
        }

        true
    }

    fn place_rock<'r>(&mut self, rock: &'r Rock) {
        let y = self.get_y(rock.y);

        if self.data.len() <= y {
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
            self.data.push(0);
        }

        for i in 0..rock.points.len() {
            let y_index = y - i;
            let mask = rock.points[i] << rock.x;

            self.data[y_index] |= mask;
        }
    }

    fn with_capacity(n: usize) -> Tower {
        Tower { data: vec![0; n] }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Rock<'s> {
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    points: &'s [u8],
}

impl<'s> Rock<'s> {
    #[inline]
    fn at(&'s self, x: i64, y: i64) -> Self {
        let mut r = self.clone();
        r.x = x;
        r.y = y;
        r
    }

    const fn new(width: i64, height: i64, points: &'s [u8]) -> Rock<'s> {
        Rock {
            x: 0,
            y: 0,
            width,
            height,
            points,
        }
    }

    fn simulate_fall<'j, 'r, I>(top_y: i64, jets: &'j mut I, tower: &'r Tower, template: &'s Rock) -> Result<Self, usize> where I: Iterator<Item=i64> {
        let mut rock = template.at(2, top_y - template.height - 3);
        let floor_y = -rock.height + 1;
        let mut settled = false;
        let mut count = 0;

        for jet in jets {
            let prev_x = rock.x;
            count += 1;

            // Push to the side
            rock.x += jet;
            if rock.x < 0 || rock.x + rock.width > 7 || !tower.can_place_rock(&rock) {
                rock.x = prev_x;
            }

            // Push down
            rock.y += 1;
            if rock.y == floor_y || !tower.can_place_rock(&rock) {
                rock.y -= 1;
                settled = true;
                break;
            }
        }

        if settled {
            Ok(rock)
        } else {
            Err(count)
        }
    }
}

const ROCK_TEMPLATES: &'static [Rock] = &[
    Rock::new(
        // ####
        4, 1, &[0b1111],
    ),
    Rock::new(
        //  #
        // ###
        //  #
        3, 3, &[0b010, 0b111, 0b010],
    ),
    Rock::new(
        //   #
        //   #
        // ###
        3, 3, &[0b100, 0b100, 0b111],
    ),
    Rock::new(
        // #
        // #
        // #
        // #
        1, 4, &[0b1, 0b1, 0b1, 0b1],
    ),
    Rock::new(
        // ##
        // ##
        2, 2, &[0b11, 0b11],
    ),
];

#[cfg(test)]
mod tests {
    use super::*;

    const P1_EXAMPLE: &[u8] = b">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    fn simulate_fall_works_on_examples() {
        let jets = parse(P1_EXAMPLE);
        let mut jets = jets.iter().cycle().copied();

        let mut tower = Tower::with_capacity(64);
        let rock_1 = Rock::simulate_fall(0, &mut jets, &tower, &ROCK_TEMPLATES[0]).unwrap();
        tower.place_rock(&rock_1);
        let rock_2 = Rock::simulate_fall(-1, &mut jets, &tower, &ROCK_TEMPLATES[1]).unwrap();
        tower.place_rock(&rock_2);
        let rock_3 = Rock::simulate_fall(-4, &mut jets, &tower, &ROCK_TEMPLATES[2]).unwrap();
        tower.place_rock(&rock_3);
        let rock_4 = Rock::simulate_fall(-6, &mut jets, &tower, &ROCK_TEMPLATES[3]).unwrap();
        tower.place_rock(&rock_4);
        let rock_5 = Rock::simulate_fall(-7, &mut jets, &tower, &ROCK_TEMPLATES[4]).unwrap();
        tower.place_rock(&rock_5);
        let rock_6 = Rock::simulate_fall(-9, &mut jets, &tower, &ROCK_TEMPLATES[0]).unwrap();
        tower.place_rock(&rock_6);
        let rock_7 = Rock::simulate_fall(-10, &mut jets, &tower, &ROCK_TEMPLATES[1]).unwrap();
        tower.place_rock(&rock_7);

        tower.render();

        assert_eq!(rock_1, ROCK_TEMPLATES[0].at(2, -1), "first rock");
        assert_eq!(rock_2, ROCK_TEMPLATES[1].at(2, -4), "second rock");
        assert_eq!(rock_3, ROCK_TEMPLATES[2].at(0, -6), "third rock");
        assert_eq!(rock_4, ROCK_TEMPLATES[3].at(4, -7), "fourth rock");
        assert_eq!(rock_5, ROCK_TEMPLATES[4].at(4, -9), "fourth rock");
        assert_eq!(rock_6, ROCK_TEMPLATES[0].at(1, -10), "fourth rock");
        assert_eq!(rock_7, ROCK_TEMPLATES[1].at(1, -13), "fourth rock");
    }

    #[test]
    fn p1_works_on_example() {
        assert_eq!(part1(&parse(P1_EXAMPLE)), 3068);
    }
}