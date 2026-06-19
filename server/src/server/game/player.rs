//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// player (drone: pos, orientation, level, inventory)
//

use super::command::Command;
use std::collections::VecDeque;

pub const STARVE_INTERVAL_UNITS: u32 = 126;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StarveResult {
    Survived,
    Died,
    Gone,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub team: usize,
    pub x: usize,
    pub y: usize,
    pub orientation: Orientation,
    pub level: u8,
    pub food: u32,
    pub queue: VecDeque<Command>,
    pub busy: bool,
    pub current_cmd: u64,
    pub cmd_seq: u64,
}

impl Player {
    pub fn new(id: u32, team: usize, x: usize, y: usize, orientation: Orientation) -> Self {
        Player {
            id,
            team,
            x,
            y,
            orientation,
            level: 1,
            food: 10,
            queue: VecDeque::new(),
            busy: false,
            current_cmd: 0,
            cmd_seq: 0,
        }
    }
}
