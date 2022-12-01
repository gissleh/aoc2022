use smallvec::SmallVec;
use common::parse2;
use common::search::{Dijkstra, DijkstraResult};

common::day!(parse, part1, part2, 100000, 500, 100);

const MISSILE_COST: i32 = 53;
const MISSILE_DAMAGE: i32 = 4;
const DRAIN_COST: i32 = 73;
const DRAIN_DAMAGE: i32 = 2;
const DRAIN_HEAL: i32 = 2;
const SHIELD_COST: i32 = 113;
const SHIELD_DURATION: u32 = 6;
const SHIELD_ARMOR: i32 = 7;
const POISON_COST: i32 = 173;
const POISON_DURATION: u32 = 6;
const POISON_DAMAGE: i32 = 3;
const RECHARGE_COST: i32 = 229;
const RECHARGE_DURATION: u32 = 5;
const RECHARGE_MANA: i32 = 101;

#[derive(Copy, Clone)]
struct BossStats {
    dmg: i32,
    hp: i32,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
struct GameState {
    // Player turn
    player_turn: bool,

    // effects
    recharge_turns: u32,
    shield_turns: u32,
    poison_turns: u32,

    // Stats
    player_health: i32,
    player_mana: i32,
    boss_health: i32,
}

impl GameState {
    pub fn new(stats: &BossStats) -> GameState {
        GameState{
            player_turn: true,

            recharge_turns: 0,
            shield_turns: 0,
            poison_turns: 0,

            player_health: 50,
            player_mana: 500,
            boss_health: stats.hp,
        }
    }
}

fn parse(data: &[u8]) -> BossStats {
    parse2::expect_bytes(b"Hit Points: ")(data)
        .and_instead(parse2::int::<i32>)
        .and_discard(parse2::expect_byte::<b'\n'>)
        .and_discard(parse2::expect_bytes(b"Damage: "))
        .and(parse2::int::<i32>)
        .map(|(hp, dmg)| BossStats {hp, dmg})
        .unwrap()
}

fn part1(input: &BossStats) -> i32 {
    run_puzzle(input, false, GameState::new(input))
}

fn part2(input: &BossStats) -> i32 {
    run_puzzle(input, true, GameState::new(input))
}

fn run_puzzle(input: &BossStats, hard_mode: bool, initial_state: GameState) -> i32 {
    let mut d = Dijkstra::new(true, true);
    let (res, _) = d.run(initial_state, 0,  |state| {
        if state.boss_health <= 0 {
            return DijkstraResult::Success
        }
        if state.player_health <= 0 {
            return DijkstraResult::DeadEnd
        }

        let mut next_steps: SmallVec<[(i32, i32, GameState); 16]> = SmallVec::new();
        let mut next_state = *state;

        if hard_mode && state.player_turn {
            next_state.player_health -= 1;
            if next_state.player_health <= 0 {
                return DijkstraResult::DeadEnd
            }
        }

        if next_state.poison_turns > 0 {
            next_state.boss_health -= POISON_DAMAGE;
            next_state.poison_turns -= 1;
        }
        if next_state.recharge_turns > 0 {
            next_state.player_mana += RECHARGE_MANA;
            next_state.recharge_turns -= 1;
        }
        let boss_damage = if next_state.shield_turns > 0 {
            next_state.shield_turns -= 1;

            if next_state.shield_turns > 0 {
                input.dmg - SHIELD_ARMOR
            } else {
                input.dmg
            }
        } else {
            input.dmg
        };

        next_state.player_turn = !next_state.player_turn;

        if state.player_turn {
            if next_state.player_mana > MISSILE_COST {
                let mut spell = next_state;
                spell.boss_health -= MISSILE_DAMAGE;
                spell.player_mana -= MISSILE_COST;

                next_steps.push((MISSILE_COST, 0, spell));
            }
            if next_state.player_mana > DRAIN_COST {
                let mut spell = next_state;
                spell.player_health += DRAIN_HEAL;
                spell.boss_health -= DRAIN_DAMAGE;
                spell.player_mana -= DRAIN_COST;

                next_steps.push((DRAIN_COST, 0, spell));
            }
            if next_state.player_mana > SHIELD_COST {
                let mut spell = next_state;
                spell.shield_turns = SHIELD_DURATION;
                spell.player_mana -= SHIELD_COST;

                next_steps.push((SHIELD_COST, 0, spell));
            }
            if next_state.player_mana > RECHARGE_COST {
                let mut spell = next_state;
                spell.recharge_turns = RECHARGE_DURATION;
                spell.player_mana -= RECHARGE_COST;

                next_steps.push((RECHARGE_COST, 0, spell));
            }
            if next_state.player_mana > POISON_COST {
                let mut spell = next_state;
                spell.poison_turns = POISON_DURATION;
                spell.player_mana -= POISON_COST;

                next_steps.push((POISON_COST, 0, spell));
            }

            if next_steps.len() == 0 {
                return DijkstraResult::DeadEnd
            }
        } else {
            if next_state.boss_health > 0 {
                next_state.player_health -= boss_damage;
            }

            next_steps.push((0, 0, next_state));
        }

        DijkstraResult::Continue(next_steps)
    }).unwrap();

    res
}

#[test]
fn test_part1() {
    let input = BossStats{
        hp: 13,
        dmg: 8,
    };

    let mut initial_stats = GameState::new(&input);
    initial_stats.player_health = 10;
    initial_stats.player_mana = 250;

    assert_eq!(run_puzzle(&input, false, initial_stats), POISON_COST + MISSILE_COST);

    initial_stats.boss_health = 14;

    assert_eq!(run_puzzle(&input, false, initial_stats), RECHARGE_COST + SHIELD_COST + DRAIN_COST + POISON_COST + MISSILE_COST);
}
