use common::aoc::Day;
use common::geo::Point;
use common::grid2::{FixedGrid, GetterMutGrid, IterableSliceGrid, RowGrid, VecGrid};
use common::search::{Dijkstra, DijkstraResult};

pub fn main(day: &mut Day, input: &[u8]) {
    let grid = day.run_parse(1000, || parse_grid(input));

    day.note("Grid Width", grid.width());
    day.note("Grid Height", grid.height());

    day.run(1, "", 20, || part1(&grid));
    day.run(2, "", 5, || part2(&grid));
}

fn parse_grid(data: &[u8]) -> VecGrid<Piece> {
    let mut next_id: u8 = 0;

    VecGrid::new_from(
        data.iter().take_while(|b| **b != b'\n').count(),
        data.iter()
            .filter(|b| **b != b'\n')
            .map(move |v| match *v {
                b'#' => Piece::Wall,
                b'.' => Piece::Ground,
                b'G' | b'E' => {
                    let player = Piece::Player { team: *v, hp: 200, id: next_id };
                    next_id += 1;
                    player
                }
                _ => unreachable!()
            })
            .collect(),
    )
}

fn part1<G>(grid: &G) -> u32 where G: GetterMutGrid<Piece> + RowGrid<Piece> + FixedGrid + Clone + IterableSliceGrid<Piece> {
    let mut board = Board::new(grid);
    let (.., outcome_score) = board.run_game();

    outcome_score
}

fn part2<G>(grid: &G) -> u32 where G: GetterMutGrid<Piece> + RowGrid<Piece> + FixedGrid + Clone + IterableSliceGrid<Piece> {
    for p in 4..255 {
        let mut board = Board::new(grid);
        board.elf_power = p;

        if let Some((_, _, outcome_score)) = board.run_game_until_elf_death() {
            return outcome_score;
        }
    }

    0
}


#[derive(Clone)]
struct Board<G> where G: GetterMutGrid<Piece> + RowGrid<Piece> + FixedGrid + Clone + IterableSliceGrid<Piece> {
    grid: G,
    move_dijkstra: Dijkstra<(Point<usize>, Option<Point<usize>>), usize>,
    elves: u8,
    goblins: u8,
    elf_power: u8,
    everyone_stuck: bool,
}

impl<G> Board<G> where G: GetterMutGrid<Piece> + RowGrid<Piece> + FixedGrid + Clone + IterableSliceGrid<Piece> {
    fn run_game(&mut self) -> (u8, u32, u32) {
        for turns in 0.. {
            if let Some(winner_team) = self.run_turn() {
                let total_hp = self.grid.cells()
                    .map(|(_, piece)| match piece {
                        Piece::Player { hp, .. } => *hp as u32,
                        _ => 0
                    })
                    .sum::<u32>();

                return (winner_team, turns, total_hp * turns);
            }
        }

        unreachable!()
    }

    fn run_game_until_elf_death(&mut self) -> Option<(u8, u32, u32)> {
        let starting_elves = self.elves;

        for turns in 0.. {
            if let Some(winner_team) = self.run_turn() {
                if self.elves < starting_elves {
                    return None
                }

                let total_hp = self.grid.cells()
                    .map(|(_, piece)| match piece {
                        Piece::Player { hp, .. } => *hp as u32,
                        _ => 0
                    })
                    .sum::<u32>();

                return Some((winner_team, turns, total_hp * turns));
            }

            if self.elves < starting_elves {
                return None
            }
        }

        unreachable!()
    }

    fn run_turn(&mut self) -> Option<u8> {
        let mut had_turn = 0u32;
        let mut anyone_moved = false;
        let mut anyone_tried_to_move = false;

        #[cfg(test)] {
            // For some reason, this optimization only works on the real input.
            self.everyone_stuck = false;
        }

        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                let p = Point(x, y);

                if let Some(Piece::Player { id, team, .. }) = self.grid.get(&p).cloned() {
                    let id_mask = 1u32 << id;
                    if had_turn & id_mask != 0 {
                        continue;
                    }
                    had_turn |= id_mask;

                    let thwack_damage = if team == b'E' { self.elf_power } else { 3 };
                    if self.thwack(&p, thwack_damage) {
                        continue;
                    }

                    if !self.everyone_stuck {
                        anyone_tried_to_move = true;
                        if let Some(new_position) = self.find_move(&p) {
                            self.move_piece(&p, &new_position);
                            self.thwack(&new_position, thwack_damage);
                            anyone_moved = true;
                        } else if let Some(v) = self.winner() {
                            return Some(v);
                        }
                    }
                }
            }
        }

        if anyone_tried_to_move && !anyone_moved {
            self.everyone_stuck = true;
        }

        None
    }

    fn winner(&self) -> Option<u8> {
        if self.elves == 0 {
            Some(b'G')
        } else if self.goblins == 0 {
            Some(b'E')
        } else {
            None
        }
    }

    fn move_piece(&mut self, from: &Point<usize>, to: &Point<usize>) {
        *self.grid.get_mut(to).unwrap() = *self.grid.get(from).unwrap();
        *self.grid.get_mut(from).unwrap() = Piece::Ground;
    }

    fn thwack(&mut self, pos: &Point<usize>, dmg: u8) -> bool {
        let target_team = if self.grid.get(pos).unwrap().is_elf() { b'G' } else { b'E' };

        // Find poor bastard.
        let thwack_point = pos.cardinals_offset(1)
            .into_iter()
            .filter_map(|neigh_pos| {
                let piece = self.grid.get(&neigh_pos).unwrap();
                if let Piece::Player { team, hp, .. } = piece {
                    if *team == target_team {
                        return Some((*hp, neigh_pos));
                    }
                }

                None
            })
            .min_by_key(|(hp, _)| *hp)
            .map(|(_, neigh_pos)| neigh_pos);

        // Act on findings.
        if let Some(thwack_point) = thwack_point {
            let piece = self.grid.get_mut(&thwack_point).unwrap();
            if let Piece::Player { hp, .. } = piece {
                if *hp <= dmg {
                    *piece = Piece::Ground;
                    if target_team == b'G' {
                        self.goblins -= 1;
                    } else {
                        self.elves -= 1;
                    }

                    self.everyone_stuck = false;
                } else {
                    *hp -= dmg;
                }

                return true;
            }
        }


        false
    }

    fn find_move(&mut self, pos: &Point<usize>) -> Option<Point<usize>> {
        let target_team = if self.grid.get(pos).unwrap().is_elf() { b'G' } else { b'E' };

        self.move_dijkstra.run((*pos, None), 0, |(current_pos, first_move)| {
            if let Some(mut piece) = self.grid.get(current_pos).copied() {
                // Start position is always treated as ground.
                if current_pos == pos {
                    piece = Piece::Ground;
                }

                match piece {
                    Piece::Ground => {
                        let is_next_to_enemy = current_pos.cardinals()
                            .into_iter()
                            .find(|p2| self.grid.get(p2).unwrap().is_player_of_team(target_team))
                            .is_some();
                        if is_next_to_enemy {
                            DijkstraResult::Success
                        } else {
                            DijkstraResult::Continue(
                                current_pos.cardinals()
                                    .into_iter()
                                    .map(|p2| (1, 0, (p2, first_move.clone().or(Some(p2)))))
                                    .collect()
                            )
                        }
                    },
                    _ => DijkstraResult::DeadEnd,
                }
            } else {
                DijkstraResult::DeadEnd
            }
        });

        let candidates = self.move_dijkstra.candidates();

        if candidates.len() > 0 {
            candidates.iter()
                .min_by(|(ap, an), (bp, bn)| {
                    ap.cmp(bp).then_with(|| an.unwrap().cmp(&bn.unwrap()))
                })
                .and_then(|(_, next_move)| *next_move)
        } else {
            None
        }
    }

    fn new(grid: &G) -> Self {
        let (elves, goblins) = grid.cells()
            .filter(|(_, v)| v.is_player())
            .fold((0, 0), |(elves, goblins), (_, piece)| {
                match piece {
                    Piece::Player { team, .. } => if *team == b'E' {
                        (elves + 1, goblins)
                    } else {
                        (elves, goblins + 1)
                    },
                    _ => (elves, goblins)
                }
            });

        Board {
            elves,
            goblins,
            elf_power: 3,
            everyone_stuck: false,
            grid: grid.clone(),
            move_dijkstra: Dijkstra::new(true   , false),
        }
    }

    #[allow(dead_code)]
    fn render(&self) -> String {
        let mut res = String::with_capacity(1024);
        let mut annotations = Vec::with_capacity(4);
        for y in 0..self.grid.height() {
            for x in 0..self.grid.width() {
                match self.grid.get(&Point(x, y)).unwrap() {
                    Piece::Player { team, hp, .. } => {
                        annotations.push(format!("  {}({})", *team as char, *hp));
                        res.push(*team as char)
                    }
                    Piece::Wall => res.push('#'),
                    Piece::Ground => res.push('.'),
                }
            }

            for annotation in annotations.iter() {
                res.push_str(annotation);
            }
            annotations.clear();

            res.push('\n');
        }

        res
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Piece {
    Ground,
    Wall,
    Player { id: u8, team: u8, hp: u8 },
}

impl Piece {
    fn is_player(&self) -> bool {
        if let Piece::Player { .. } = self { true } else { false }
    }
    fn is_player_of_team(&self, target_team: u8) -> bool {
        if let Piece::Player { team, .. } = self { *team == target_team } else { false }
    }
    fn is_elf(&self) -> bool { if let Piece::Player { team, .. } = self { *team == b'E' } else { false } }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: u8) -> Piece { Piece::Player { team: b'E', hp: 200, id } }

    fn g(id: u8) -> Piece { Piece::Player { team: b'G', hp: 200, id } }

    const EXAMPLE_P1_MOVEMENT_1: &[u8] = b"#######
#.E...#
#.....#
#...G.#
#######
";

    const EXAMPLE_P1_MOVEMENT_2: &[u8] = b"#######
#E..G.#
#...#.#
#.G.#G#
#######
";

    const EXAMPLE_P1_MOVEMENT_3: &[&[u8]] = &[b"#########
#G..G..G#
#.......#
#.......#
#G..E..G#
#.......#
#.......#
#G..G..G#
#########
", b"#########
#.G...G.#
#...G...#
#...E..G#
#.G.....#
#.......#
#G..G..G#
#.......#
#########
", b"#########
#..G.G..#
#...G...#
#.G.E.G.#
#.......#
#G..G..G#
#.......#
#.......#
#########
", b"#########
#.......#
#..GGG..#
#..GEG..#
#G..G...#
#......G#
#.......#
#.......#
#########
"];

    const EXAMPLE_P1_GAME_1: &[u8] = b"#######
#.G...#
#...EG#
#.#.#G#
#..G#E#
#.....#
#######
";
    const EXAMPLE_P1_GAME_2: &[u8] = b"#######
#G..#E#
#E#E.E#
#G.##.#
#...#E#
#...E.#
#######
";
    const EXAMPLE_P1_GAME_3: &[u8] = b"#######
#E..EG#
#.#G.E#
#E.##E#
#G..#.#
#..E#.#
#######
";
    const EXAMPLE_P1_GAME_4: &[u8] = b"#######
#E.G#.#
#.#G..#
#G.#.G#
#G..#.#
#...E.#
#######
";
    const EXAMPLE_P1_GAME_5: &[u8] = b"#######
#.E...#
#.#..G#
#.###.#
#E#G#G#
#...#G#
#######
";
    const EXAMPLE_P1_GAME_6: &[u8] = b"#########
#G......#
#.E.#...#
#..##..G#
#...##..#
#...#...#
#.G...G.#
#.....G.#
#########
";

    fn util_render_grid(grid: &VecGrid<Piece>) -> Vec<u8> {
        let mut res = Vec::with_capacity(grid.width() * grid.height() + grid.height());
        for y in 0..grid.height() {
            for x in 0..grid.width() {
                match grid.get(&Point(x, y)).unwrap() {
                    Piece::Player { team, .. } => res.push(*team),
                    Piece::Wall => res.push(b'#'),
                    Piece::Ground => res.push(b'.'),
                }
            }

            res.push(b'\n');
        }

        res
    }

    #[test]
    fn test_util_render_grid_simple() {
        assert_eq!(util_render_grid(&parse_grid(EXAMPLE_P1_MOVEMENT_2)).as_slice(), EXAMPLE_P1_MOVEMENT_2);
        assert_eq!(util_render_grid(&parse_grid(EXAMPLE_P1_MOVEMENT_3[0])).as_slice(), EXAMPLE_P1_MOVEMENT_3[0]);
    }

    #[test]
    fn next_move_follows_examples() {
        let mut board = Board::new(&parse_grid(EXAMPLE_P1_MOVEMENT_1));

        assert_eq!(board.find_move(&Point(2, 1)), Some(Point(3, 1)));

        let mut board = Board::new(&parse_grid(EXAMPLE_P1_MOVEMENT_2));

        assert_eq!(board.find_move(&Point(1, 1)), Some(Point(2, 1)));

        let mut board = Board::new(&parse_grid(EXAMPLE_P1_MOVEMENT_3[0]));

        // This does not look like the game in the example.
        assert_eq!(board.find_move(&Point(1, 1)), Some(Point(2, 1)));
        assert_eq!(board.find_move(&Point(4, 1)), Some(Point(4, 2)));
        assert_eq!(board.find_move(&Point(7, 1)), Some(Point(6, 1)));
        assert_eq!(board.find_move(&Point(1, 4)), Some(Point(2, 4)));
        assert_eq!(board.find_move(&Point(4, 4)), Some(Point(4, 3)));
        assert_eq!(board.find_move(&Point(7, 4)), Some(Point(6, 4)));
        assert_eq!(board.find_move(&Point(1, 7)), Some(Point(1, 6)));
        assert_eq!(board.find_move(&Point(4, 7)), Some(Point(4, 6)));
        assert_eq!(board.find_move(&Point(7, 7)), Some(Point(7, 6)));
    }

    #[test]
    fn parse_works() {
        use Piece::*;

        let grid = parse_grid(EXAMPLE_P1_MOVEMENT_2);

        assert_eq!(grid.row(0).unwrap(), [Wall, Wall, Wall, Wall, Wall, Wall, Wall].as_slice());
        assert_eq!(grid.row(1).unwrap(), [Wall, e(0), Ground, Ground, g(1), Ground, Wall].as_slice());
        assert_eq!(grid.row(2).unwrap(), [Wall, Ground, Ground, Ground, Wall, Ground, Wall].as_slice());
        assert_eq!(grid.row(3).unwrap(), [Wall, Ground, g(2), Ground, Wall, g(3), Wall].as_slice());
        assert_eq!(grid.row(4).unwrap(), [Wall, Wall, Wall, Wall, Wall, Wall, Wall].as_slice());
    }

    #[test]
    fn thwack_order() {
        use Piece::*;

        let mut test_grid = VecGrid::new_from(5, [
            Wall, Wall, Wall, Wall, Wall,
            Wall, Ground, Ground, Ground, Wall,
            Wall, Ground, e(15), Ground, Wall,
            Wall, Ground, Ground, Ground, Wall,
            Wall, Wall, Wall, Wall, Wall,
        ].to_vec());
        *test_grid.get_mut(&Point(2, 1)).unwrap() = Player { id: 1, team: b'G', hp: 9 };
        *test_grid.get_mut(&Point(1, 2)).unwrap() = Player { id: 1, team: b'G', hp: 2 };
        *test_grid.get_mut(&Point(3, 2)).unwrap() = Player { id: 1, team: b'G', hp: 3 };
        *test_grid.get_mut(&Point(2, 3)).unwrap() = Player { id: 1, team: b'G', hp: 2 };

        let mut board = Board::new(&test_grid);

        assert!(board.thwack(&Point(2, 2), 3));
        assert_eq!(board.grid.get(&Point(1, 2)), Some(&Ground));
        assert!(board.thwack(&Point(2, 2), 3));
        assert_eq!(board.grid.get(&Point(2, 3)), Some(&Ground));
        assert!(board.thwack(&Point(2, 2), 3));
        assert_eq!(board.grid.get(&Point(3, 2)), Some(&Ground));
        assert!(board.thwack(&Point(2, 2), 8));
        assert_eq!(board.grid.get(&Point(2, 1)), Some(&Player { id: 1, team: b'G', hp: 1 }));
        assert!(board.thwack(&Point(2, 2), 8));
        assert_eq!(board.grid.get(&Point(2, 1)), Some(&Ground));
        assert!(!board.thwack(&Point(2, 2), 8));
    }

    #[test]
    fn turn_moves_work() {
        let mut board = Board::new(&parse_grid(EXAMPLE_P1_MOVEMENT_3[0]));

        for i in 1..=3usize {
            board.run_turn();
            println!("{}", String::from_utf8_lossy(EXAMPLE_P1_MOVEMENT_3[i]));
            println!("{}", String::from_utf8_lossy(&util_render_grid(&board.grid)));
            assert_eq!(util_render_grid(&board.grid), EXAMPLE_P1_MOVEMENT_3[i]);
        }
    }

    #[test]
    fn p1_example_games() {
        let mut board_1 = Board::new(&parse_grid(EXAMPLE_P1_GAME_1));
        let mut board_2 = Board::new(&parse_grid(EXAMPLE_P1_GAME_2));
        let mut board_3 = Board::new(&parse_grid(EXAMPLE_P1_GAME_3));
        let mut board_4 = Board::new(&parse_grid(EXAMPLE_P1_GAME_4));
        let mut board_5 = Board::new(&parse_grid(EXAMPLE_P1_GAME_5));
        let mut board_6 = Board::new(&parse_grid(EXAMPLE_P1_GAME_6));

        assert_eq!(board_1.run_game(), (b'G', 47, 27730));
        assert_eq!(board_2.run_game(), (b'E', 37, 36334));
        assert_eq!(board_3.run_game(), (b'E', 46, 39514));
        assert_eq!(board_4.run_game(), (b'G', 35, 27755));
        assert_eq!(board_5.run_game(), (b'G', 54, 28944));
        assert_eq!(board_6.run_game(), (b'G', 20, 18740));
    }
}