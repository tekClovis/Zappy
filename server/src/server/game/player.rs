//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// player (drone: pos, orientation, level, inventory)
//

use super::command::Command;
use super::map::{RESOURCE_COUNT, Resource};
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
    pub inventory: [u32; RESOURCE_COUNT],
    pub queue: VecDeque<Command>,
    pub busy: bool,
    pub current_cmd: u64,
    pub cmd_seq: u64,
}

impl Player {
    pub fn new(id: u32, team: usize, x: usize, y: usize, orientation: Orientation) -> Self {
        let mut inventory = [0u32; RESOURCE_COUNT];
        inventory[Resource::Food as usize] = 10;
        Player {
            id,
            team,
            x,
            y,
            orientation,
            level: 1,
            inventory,
            queue: VecDeque::new(),
            busy: false,
            current_cmd: 0,
            cmd_seq: 0,
        }
    }

    pub fn inventory_string(&self) -> String {
        let body = Resource::ALL
            .iter()
            .map(|r| format!("{} {}", r.name(), self.inventory[*r as usize]))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{body}]")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_player_inventory_string() {
        let p = Player::new(1, 0, 0, 0, Orientation::North);
        assert_eq!(
            p.inventory_string(),
            "[food 10, linemate 0, deraumere 0, sibur 0, mendiane 0, phiras 0, thystame 0]"
        );
    }
}
