//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Scheduler
//

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    ActionDone { player: u32, command_id: u64 },
    RespawnResources,
    IncantationDone { tile: (usize, usize), level: u8 },
    Starve { player: u32 },
}

struct Scheduled {
    at: Instant,
    event: Event,
}

impl PartialEq for Scheduled {
    fn eq(&self, other: &Self) -> bool {
        self.at == other.at
    }
}
impl Eq for Scheduled {}
impl PartialOrd for Scheduled {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Scheduled {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.at.cmp(&other.at)
    }
}

#[derive(Default)]
pub struct Scheduler {
    heap: BinaryHeap<Reverse<Scheduled>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub fn schedule_units(&mut self, cost: u32, f: u32, event: Event) {
        let secs = cost as f64 / f as f64;
        self.schedule_at(Instant::now() + Duration::from_secs_f64(secs), event);
    }

    pub fn schedule_at(&mut self, at: Instant, event: Event) {
        self.heap.push(Reverse(Scheduled { at, event }));
    }

    pub fn next_timeout(&self) -> Option<Duration> {
        self.heap
            .peek()
            .map(|Reverse(s)| s.at.saturating_duration_since(Instant::now()))
    }

    pub fn pop_due(&mut self, now: Instant) -> Option<Event> {
        match self.heap.peek() {
            Some(Reverse(s)) if s.at <= now => self.heap.pop().map(|Reverse(s)| s.event),
            _ => None,
        }
    }
}
