use std::cmp::max;
use arrayvec::ArrayVec;
use num::range_step;
use common::aoc::Day;
use common::geo::Point;
use common::grid2::{FixedGrid, GetterGrid, GetterMutGrid, SubGrid, VecGrid};
use common::parse3;
use common::parse3::{choice, Parser, unsigned_int};

const RIGHT: i8 = 0;
const DOWN: i8 = 1;
const LEFT: i8 = 2;
const UP: i8 = 3;
const NO_NEIGHBOR: u8 = 6;
const TURN_LEFT: i8 = -1;
const TURN_RIGHT: i8 = 1;
const NO_TURN: i8 = 0;

pub fn main(day: &mut Day, input: &[u8]) {
    let (grid, instructions) = day.run_parse(1000, || parse(input));

    day.note("Grid Width", grid.width());
    day.note("Grid Height", grid.height());
    day.note("Instructions", instructions.len());

    day.run(1, "", 10000, || part1(&grid, &instructions));
    day.run(2, "", 100, || part2(&grid, 50, &instructions));
}

fn parse(data: &[u8]) -> (VecGrid<u8>, Vec<(u8, i8)>) {
    let mut grid_width = 0;
    let mut grid_height = 0;
    for line in parse3::line().iterate(data) {
        if line.len() == 0 {
            break;
        }

        grid_width = max(grid_width, line.len());
        grid_height += 1;
    }

    let mut grid = VecGrid::new_with(grid_width, grid_height, b' ');
    let mut p = Point(0, 0);
    for ch in data {
        match *ch {
            b'\n' => {
                p.1 += 1;
                if p.1 == grid_height {
                    break;
                }
                p.0 = 0;
            }
            b'#' | b'.' => {
                *grid.get_mut(&p).unwrap() = *ch;
                p.0 += 1;
            }
            _ => { p.0 += 1 }
        }
    }

    let (_, pos) = b"\n\n".first_parsable(data).unwrap();
    let instructions = unsigned_int::<u8>()
        .and(choice([
            b'L'.map_to_value(TURN_LEFT),
            b'R'.map_to_value(TURN_RIGHT),
            b'\n'.map_to_value(NO_TURN),
        ]))
        .repeat()
        .parse(&data[pos + 2..])
        .unwrap();

    (grid, instructions)
}

fn part1<G>(grid: &G, instructions: &[(u8, i8)]) -> usize where G: GetterMutGrid<u8> + FixedGrid {
    let mut pos = Point(0usize, 0usize);
    while *grid.get(&pos).unwrap() != b'.' {
        pos.0 += 1;
    }

    let right = grid.width() - 1;
    let bottom = grid.height() - 1;

    let mut dir = RIGHT;
    for (steps, turn) in instructions {
        let mut steps_left = *steps;
        let mut new_pos = pos;

        while steps_left > 0 {
            new_pos = move_points(&new_pos, dir, right, bottom);
            match *grid.get(&new_pos).unwrap() {
                b'.' => {
                    pos = new_pos;
                    steps_left -= 1;
                }
                b' ' => {}
                b'#' => { break; }
                ch => panic!("unknown grid cell {}", ch)
            }
        }

        dir = (dir + turn) % 4;
        if dir == -1 {
            dir = 3;
        }
    }

    password(&pos, dir)
}

fn part2<G>(grid: &G, side_len: usize, instructions: &[(u8, i8)]) -> usize where G: GetterGrid<u8> + FixedGrid {
    let mut cube = Cube::new(grid, side_len);
    cube.connect_adjacent_faces();
    cube.fold_once();

    let (pos, dir) = cube.run(0, Point(0, 0), instructions);
    password(&pos, dir)
}

fn password(pos: &Point<usize>, dir: i8) -> usize {
    ((pos.1 + 1) * 1000)
        + ((pos.0 + 1) * 4)
        + dir as usize
}

fn move_points(p: &Point<usize>, dir: i8, width: usize, height: usize) -> Point<usize> {
    match dir % 4 {
        LEFT => Point(if p.0 == 0 { width } else { p.0 - 1 }, p.1),
        UP => Point(p.0, if p.1 == 0 { height } else { p.1 - 1 }),
        RIGHT => Point(if p.0 == width { 0 } else { p.0 + 1 }, p.1),
        DOWN => Point(p.0, if p.1 == height { 0 } else { p.1 + 1 }),
        _ => panic!("Bad direction")
    }
}

struct Cube<'i, G> where G: GetterGrid<u8> {
    faces: ArrayVec<CubeFace<'i, G>, 6>,
}

impl<'i, G> Cube<'i, G> where G: GetterGrid<u8> + FixedGrid {
    fn run(&self, face_index: u8, face_pos: Point<usize>, instructions: &[(u8, i8)]) -> (Point<usize>, i8) {
        let mut cursor = self.cursor_on(face_index, face_pos);
        cursor.run_multiple(instructions);

        #[cfg(test)] println!("run completed at faces[{}][{},{}] facing {}",
                              cursor.face_index, cursor.face_pos.0, cursor.face_pos.1, cursor.dir);

        (cursor.super_pos(), cursor.dir)
    }

    #[allow(dead_code)]
    fn cursor(&'i self) -> CubeCursor<'i, G> {
        self.cursor_on(0, Point(0, 0))
    }

    fn cursor_on(&'i self, face_index: u8, face_pos: Point<usize>) -> CubeCursor<'i, G> {
        CubeCursor {
            cube: self,
            dir: RIGHT,
            face_index,
            face_pos,
        }
    }

    fn connect_adjacent_faces(&mut self) {
        const DIRECTIONS: &[usize; 4] = &[3, 2, 0, 1];

        for i in 0..6 {
            let cardinals = self.faces[i].index.cardinals_wrapping(3, 3);

            for (j, adj_index) in cardinals.into_iter().enumerate() {
                if let Some(index) = self.faces.iter().position(|v| v.index == adj_index) {
                    self.faces[i].neighbors[DIRECTIONS[j]] = (index as u8, DIRECTIONS[j] as i8)
                }
            }
        }
    }

    fn fold_once(&mut self) -> bool {
        const TABLE: &[(usize, &[usize], i8)] = &[
            (0, &[1, 0], DOWN), // right = down right
            (0, &[3, 0], UP), // right = up right
            (0, &[2, 2, 2], RIGHT), // right = left left left
            (0, &[1, 1, 0], LEFT), // right = down down right
            (0, &[3, 3, 0], LEFT), // right = up up right
            (0, &[2, 3, 3], LEFT), // right = left up up
            (0, &[2, 1, 1], LEFT), // right = left down down
            (1, &[0, 1], RIGHT), // down = right down
            (1, &[2, 1], LEFT), // down = left down
            (1, &[3, 3, 3], DOWN), // down = up up up
            (1, &[0, 0, 1], UP), // down = right right down
            (1, &[2, 2, 1], UP), // down = left left down
            (1, &[3, 2, 2], UP), // down = up left left
            (1, &[3, 0, 3, 3, 0], DOWN), // down = up right up up right
            (1, &[2, 3, 2, 2], RIGHT), // down = up right up up right
            (2, &[1, 2], DOWN), // left = down left
            (2, &[3, 2], UP), // left = up left
            (2, &[0, 0, 0], RIGHT), // left = right right right
            (2, &[1, 1, 2], RIGHT), // left = down down left
            (2, &[0, 3, 3], RIGHT), // left = right up up
            (2, &[0, 0, 1, 0], UP), // left = right right down right
            (2, &[3, 0, 3, 3], DOWN), // left = right right down right
            (3, &[0, 3], RIGHT), // up = right up
            (3, &[2, 3], LEFT), // up = left up
            (3, &[1, 2, 2], DOWN), // up = down left left
            (3, &[1, 0, 0], DOWN), // up = down right right
            (3, &[0, 0, 3], DOWN), // up = right right up
            (3, &[2, 2, 3], DOWN), // up = left left up
            (3, &[1, 0, 0], UP), // up = down down down
            (3, &[2, 1, 1, 2, 1], UP), // up = left down down left down
            (3, &[1, 1, 2, 1], RIGHT), // down = down down left down
        ];

        let mut any_folded = false;
        let mut faces = self.faces.clone();
        for i in 0..6 {
            'fold_table_loop: for (ni, list, mut dir) in TABLE.iter().cloned() {
                if faces[i].neighbors[ni].0 != 6 {
                    continue; // Already folded
                }

                let mut curr = i;
                for index in list {
                    let (n, d) = self.faces[curr].neighbors[*index];
                    if n == 6 {
                        continue 'fold_table_loop; // Cannot fold this way
                    }

                    let diff = d - (*index as i8);
                    dir = (4 - diff + dir) % 4;

                    curr = n as usize;
                }

                #[cfg(test)] println!("[{}].[{}] with {:?} = {}", i, ni, list, curr);

                any_folded = true;
                faces[i].neighbors[ni] = (curr as u8, dir);
            }
        }
        self.faces = faces;

        any_folded
    }

    fn new(grid: &'i G, side_len: usize) -> Self {
        let mut faces: ArrayVec<_, 6> = ArrayVec::new();
        for y in range_step(0, grid.height(), side_len) {
            for x in range_step(0, grid.width(), side_len) {
                if *grid.get(&Point(x, y)).unwrap() == b' ' {
                    continue;
                }

                faces.push(CubeFace {
                    index: Point(x / side_len, y / side_len),
                    grid: SubGrid::new(
                        grid, Point(x, y),
                        side_len, side_len,
                    ),
                    neighbors: [(NO_NEIGHBOR, -1); 4],
                });
            }
        }

        Self { faces }
    }
}

struct CubeFace<'i, G> where G: GetterGrid<u8> {
    index: Point<usize>,
    grid: SubGrid<'i, G>,
    neighbors: [(u8, i8); 4],
}

impl<'i, G> Clone for CubeFace<'i, G> where G: GetterGrid<u8> {
    fn clone(&self) -> Self {
        Self{
            index: self.index,
            grid: self.grid,
            neighbors: self.neighbors,
        }
    }
}

struct CubeCursor<'i, G> where G: GetterGrid<u8> {
    cube: &'i Cube<'i, G>,
    dir: i8,
    face_index: u8,
    face_pos: Point<usize>,
}

impl<'i, G> Copy for CubeCursor<'i, G> where G: GetterGrid<u8> {}

impl<'i, G> Clone for CubeCursor<'i, G> where G: GetterGrid<u8> {
    #[inline]
    fn clone(&self) -> Self {
        Self {
            cube: self.cube,
            dir: self.dir,
            face_index: self.face_index,
            face_pos: self.face_pos,
        }
    }
}

impl<'i, G> CubeCursor<'i, G> where G: GetterGrid<u8> {
    fn super_pos(&self) -> Point<usize> {
        self.cube.faces[self.face_index as usize].grid.super_pos(&self.face_pos)
    }

    fn run_multiple(&mut self, instructions: &[(u8, i8)]) {
        for i in instructions {
            self.run(*i);
        }
    }

    fn run(&mut self, instruction: (u8, i8)) {
        let (steps, turn) = instruction;

        for _ in 0..steps {
            match self.step() {
                Some(v) => { *self = v }
                None => { break; }
            }
        }

        self.dir = (self.dir + turn) % 4;
        if self.dir == -1 {
            self.dir = 3;
        }

        #[cfg(test)] println!("turned {} to {}", turn, self.dir);
    }

    fn step(&self) -> Option<Self> {
        let face = &self.cube.faces[self.face_index as usize];
        let mut next = *self;
        let far_side = face.grid.width() - 1;

        match self.dir {
            RIGHT => {
                if next.face_pos.0 == far_side {
                    next.face_index = face.neighbors[0].0;
                    next.dir = face.neighbors[0].1;
                    next.face_pos = Point(0, self.face_pos.1);
                } else {
                    next.face_pos = Point(self.face_pos.0 + 1, self.face_pos.1);
                }
            }
            DOWN => {
                if next.face_pos.1 == far_side {
                    next.face_index = face.neighbors[1].0;
                    next.dir = face.neighbors[1].1;
                    next.face_pos = Point(self.face_pos.0, 0);
                } else {
                    next.face_pos = Point(self.face_pos.0, self.face_pos.1 + 1);
                }
            }
            LEFT => {
                if next.face_pos.0 == 0 {
                    next.face_index = face.neighbors[2].0;
                    next.dir = face.neighbors[2].1;
                    next.face_pos = Point(far_side, self.face_pos.1);
                } else {
                    next.face_pos = Point(self.face_pos.0 - 1, self.face_pos.1);
                }
            }
            UP => {
                if next.face_pos.1 == 0 {
                    next.face_index = face.neighbors[3].0;
                    next.dir = face.neighbors[3].1;
                    next.face_pos = Point(self.face_pos.0, far_side);
                } else {
                    next.face_pos = Point(self.face_pos.0, self.face_pos.1 - 1);
                }
            }
            _ => panic!("CubeCursor::step: unknown direction {}", self.dir)
        }

        // If the move changed the direction
        if self.dir != next.dir {
            match (self.dir, next.dir) {
                (LEFT, RIGHT) | (RIGHT, LEFT) | (UP, DOWN) | (DOWN, UP) => {
                    next.face_pos = Point(far_side - next.face_pos.0, far_side - next.face_pos.1);
                }
                (RIGHT, DOWN) => next.face_pos = Point(far_side - self.face_pos.1, 0),
                (DOWN, RIGHT) => next.face_pos = Point(0, far_side - self.face_pos.0),
                (UP, RIGHT) => next.face_pos = Point(0, self.face_pos.0),
                (RIGHT, UP) => next.face_pos = Point(self.face_pos.1, far_side),
                (LEFT, UP) => next.face_pos = Point(far_side - self.face_pos.1, far_side),
                (UP, LEFT) => next.face_pos = Point(far_side, far_side - self.face_pos.0),
                (DOWN, LEFT) => next.face_pos = Point(far_side, self.face_pos.0),
                (LEFT, DOWN) => next.face_pos = Point(self.face_pos.1, 0),
                _ => {}
            }
        }

        #[cfg(test)] print!("stepping to face[{}]:{} at {:?}", next.face_index, next.dir, next.face_pos);

        let next_face = &self.cube.faces[next.face_index as usize];
        if *next_face.grid.get(&next.face_pos).unwrap() == b'.' {
            #[cfg(test)] println!();
            Some(next)
        } else {
            #[cfg(test)] println!("...doh!");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use common::grid2::{render_char_grid, render_grid};
    use super::*;

    const P1_EXAMPLE: &[u8] = include_bytes!("test_fixtures/d22_p1_example.txt");

    #[test]
    fn parse_works_on_example() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        assert_eq!(grid.width(), 16);
        assert_eq!(grid.height(), 12);
        assert_eq!(instructions.as_slice(), &[
            // 10R5L5R10L4R5L5
            (10u8, 1i8),
            (5, -1),
            (5, 1),
            (10, -1),
            (4, 1),
            (5, -1),
            (5, 0),
        ]);
    }

    #[test]
    fn p1_works_on_example() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        assert_eq!(part1(&grid, &instructions), 6032);
    }

    #[test]
    fn p2_works_on_example() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        assert_eq!(part2(&grid, 4,  &instructions), 5031);
    }

    #[test]
    fn cube_new_splits_correctly() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        let cube = Cube::new(&grid, 4);

        assert_eq!(cube.faces.len(), 6);
        assert_eq!(render_char_grid(&cube.faces[0].grid), "...#\n.#..\n#...\n....\n");
        assert_eq!(render_char_grid(&cube.faces[1].grid), "...#\n....\n..#.\n....\n");
        assert_eq!(render_char_grid(&cube.faces[2].grid), "....\n....\n...#\n....\n");
        assert_eq!(render_char_grid(&cube.faces[3].grid), "...#\n#...\n....\n..#.\n");
        assert_eq!(render_char_grid(&cube.faces[4].grid), "...#\n....\n.#..\n....\n");
        assert_eq!(render_char_grid(&cube.faces[5].grid), "....\n.#..\n....\n..#.\n");
    }

    #[test]
    fn cube_walk_works_on_4_p1() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        let mut cube = Cube::new(&grid, 4);

        // "fold" the cube
        cube.faces[0].neighbors = [(0, RIGHT), (3, DOWN), (0, LEFT), (4, UP)];
        cube.faces[1].neighbors = [(2, RIGHT), (1, DOWN), (4, LEFT), (1, UP)];
        cube.faces[2].neighbors = [(3, RIGHT), (2, DOWN), (2, LEFT), (2, UP)];
        cube.faces[3].neighbors = [(1, RIGHT), (4, DOWN), (3, LEFT), (0, UP)];
        cube.faces[4].neighbors = [(5, RIGHT), (0, DOWN), (5, LEFT), (3, UP)];
        cube.faces[5].neighbors = [(4, RIGHT), (5, DOWN), (4, LEFT), (5, UP)];

        let mut res = cube.run(0, Point(0, 0), &instructions);
        assert_eq!(res, (Point(7, 5), RIGHT));
    }

    #[test]
    fn cube_walk_works_on_manually_folded_example() {
        let (grid, instructions) = parse(P1_EXAMPLE);
        let mut cube = Cube::new(&grid, 4);

        cube.faces[0].neighbors = [(5, LEFT), (3, DOWN), (2, DOWN), (1, DOWN)];
        cube.faces[1].neighbors = [(2, RIGHT), (4, UP), (5, UP), (0, DOWN)];
        cube.faces[2].neighbors = [(3, RIGHT), (4, RIGHT), (1, LEFT), (0, RIGHT)];
        cube.faces[3].neighbors = [(5, DOWN), (4, DOWN), (2, LEFT), (0, UP)];
        cube.faces[4].neighbors = [(5, RIGHT), (1, UP), (2, UP), (3, UP)];
        cube.faces[5].neighbors = [(0, LEFT), (1, RIGHT), (4, LEFT), (3, LEFT)];

        assert_eq!(
            cube.run(3, Point(2, 1), &[(4, NO_TURN)]),
            (Point(14, 10), DOWN),
        );

        assert_eq!(
            cube.run(2, Point(2, 2), &[(0, TURN_RIGHT), (4, NO_TURN)]),
            (Point(10, 9), RIGHT),
        );

        assert_eq!(
            cube.run(2, Point(2, 2), &[(0, TURN_LEFT), (4, NO_TURN)]),
            (Point(6, 4), UP),
        );

        assert_eq!(
            cube.run(2, Point(3, 1), &[(0, TURN_LEFT), (99, NO_TURN)]),
            (Point(12, 8), LEFT),
        );

        assert_eq!(
            cube.run(5, Point(2, 2), &[(0, TURN_LEFT), (99, NO_TURN)]),
            (Point(9, 5), LEFT),
        );

        assert_eq!(
            cube.run(
                2, Point(2, 2),
                &[(0, TURN_LEFT), (1, TURN_RIGHT), (1, TURN_LEFT), (4, NO_TURN)],
            ),
            (Point(10, 3), RIGHT),
        );

        assert_eq!(
            cube.run(0, Point(0, 0), &instructions),
            (Point(6, 4), UP),
        )
    }

    #[test]
    fn cube_connects_adjacent_faces() {
        const BLANK: (u8, i8) = (NO_NEIGHBOR, -1);

        let (grid, instructions) = parse(P1_EXAMPLE);
        let mut cube = Cube::new(&grid, 4);
        cube.connect_adjacent_faces();

        assert_eq!(cube.faces[0].neighbors, [BLANK, (3, DOWN), BLANK, BLANK]);
        assert_eq!(cube.faces[1].neighbors, [(2, RIGHT), BLANK, BLANK, BLANK]);
        assert_eq!(cube.faces[2].neighbors, [(3, RIGHT), BLANK, (1, LEFT), BLANK]);
        assert_eq!(cube.faces[3].neighbors, [BLANK, (4, DOWN), (2, LEFT), (0, UP)]);
        assert_eq!(cube.faces[4].neighbors, [(5, RIGHT), BLANK, BLANK, (3, UP)]);
        assert_eq!(cube.faces[5].neighbors, [BLANK, BLANK, (4, LEFT), BLANK]);
    }

    #[test]
    fn cube_fold_once_does_it_right() {
        const BLANK: (u8, i8) = (NO_NEIGHBOR, -1);

        let (grid, instructions) = parse(P1_EXAMPLE);
        let mut cube = Cube::new(&grid, 4);
        cube.connect_adjacent_faces();

        assert_eq!(cube.fold_once(), true);
        assert_eq!(cube.faces[0].neighbors, [(5, LEFT), (3, DOWN), (2, DOWN), (1, DOWN)], "0");
        assert_eq!(cube.faces[1].neighbors, [(2, RIGHT), (4, UP), (5, UP), (0, DOWN)], "1");
        assert_eq!(cube.faces[2].neighbors, [(3, RIGHT), (4, RIGHT), (1, LEFT), (0, RIGHT)], "2");
        assert_eq!(cube.faces[3].neighbors, [(5, DOWN), (4, DOWN), (2, LEFT), (0, UP)], "3");
        assert_eq!(cube.faces[4].neighbors, [(5, RIGHT), (1, UP), (2, UP), (3, UP)], "4");
        assert_eq!(cube.faces[5].neighbors, [(0, LEFT), (1, RIGHT), (4, LEFT), (3, LEFT)], "5");

        assert_eq!(cube.fold_once(), false);
    }
}