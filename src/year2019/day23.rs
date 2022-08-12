use common::intcode::{FixedMemory, Intcode, IntcodeResult, Memory};
use arrayvec::ArrayVec;

common::day!(parse, part1, part2, 1000, 100, 100);

fn part1<M>(initial_program: &M) -> i64 where M: Memory {
    let mut queue: Vec<(usize, i64, i64)> = Vec::with_capacity(8);
    let mut computers: ArrayVec<Computer<M>, 50> = (0..50).map(|i| {
        Computer::new(initial_program, i)
    }).collect();

    loop {
        for i in 0..computers.len() {
            // Check for new messages if inbox is empty
            if computers[i].inbox.is_empty() {
                if let Some(j) = queue.iter().position(|(j, ..)| *j == i) {
                    let (_, a, b) = queue.remove(j);

                    computers[i].inbox.push(b);
                    computers[i].inbox.push(a);
                }
            }

            // Run the computer for one step.
            computers[i].run();

            // Take the message if the outbox is full.
            if let Some(msg) = computers[i].get_message() {
                if msg.0 == 255 {
                    return msg.2;
                }

                queue.push(msg);
            }
        }
    }
}

fn part2<M>(initial_program: &M) -> i64 where M: Memory {
    let mut queue: Vec<(usize, i64, i64)> = Vec::with_capacity(8);
    let mut computers: ArrayVec<Computer<M>, 50> = (0..50).map(|i| {
        Computer::new(initial_program, i)
    }).collect();
    let mut last_nat = (0i64, 0i64);
    let mut last_delivered_nat = (i64::MAX, i64::MAX);

    loop {
        let mut is_idle = true;

        for i in 0..computers.len() {
            // Check for new messages if inbox is empty
            if computers[i].inbox.is_empty() {
                if let Some(j) = queue.iter().position(|(j, ..)| *j == i) {
                    let (_, a, b) = queue.remove(j);

                    computers[i].inbox.push(b);
                    computers[i].inbox.push(a);
                }
            }

            // Run the computer for one step.
            computers[i].run();

            if is_idle && !computers[i].idle {
                is_idle = false;
            }

            // Take the message if the outbox is full.
            if let Some(msg) = computers[i].get_message() {
                is_idle = false;

                if msg.0 == 255 {
                    last_nat = (msg.1, msg.2);
                } else {
                    queue.push(msg);
                }
            }
        }

        if is_idle && queue.len() == 0 {
            if last_nat == last_delivered_nat {
                return last_nat.1;
            }

            queue.push((0, last_nat.0, last_nat.1));
            last_delivered_nat = last_nat;
        }
    }
}

fn parse(input: &[u8]) -> FixedMemory<4096> {
    FixedMemory::parse(input)
}

struct Computer<M> where M: Memory {
    nic: Intcode<M>,
    outbox: ArrayVec<i64, 3>,
    inbox: ArrayVec<i64, 2>,
    idle: bool,
}

impl<M> Computer<M> where M: Memory {
    fn get_message(&mut self) -> Option<(usize, i64, i64)> {
        if self.outbox.is_full() {
            let d = self.outbox[0] as usize;
            let a = self.outbox[1];
            let b = self.outbox[2];
            self.outbox.clear();

            Some((d, a, b))
        } else {
            None
        }
    }

    fn run(&mut self) {
        match self.nic.run() {
            IntcodeResult::Output(v) => {
                self.outbox.push(v);
                if !self.outbox.is_full() {
                    self.run();
                }
            }
            IntcodeResult::InputNeeded => {
                if let Some(v) = self.inbox.pop() {
                    self.nic.run_step(Some(v));
                    self.run();
                } else {
                    self.nic.run_step(Some(-1));
                    self.idle = true;
                }
            }
            _ => {}
        }
    }

    fn new(initial_program: &M, id: i64) -> Self {
        let mut c = Self {
            inbox: ArrayVec::default(),
            outbox: ArrayVec::default(),
            nic: Intcode::new(initial_program),
            idle: false,
        };
        c.inbox.push(id);

        c
    }
}
