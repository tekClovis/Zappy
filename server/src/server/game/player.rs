//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// player (drone: pos, orientation, level, inventory)
//

/// Facing direction. The discriminants are the codes the GUI protocol uses
/// (`O` in `pnw #n X Y O L N`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

/// One AI-controlled drone. Bound to a `Team` (by index into `World.teams`).
#[allow(dead_code)] // fields are read once command execution + GUI events land
#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub team: usize,
    pub x: usize,
    pub y: usize,
    pub orientation: Orientation,
    pub level: u8,
    pub food: u32,
}

impl Player {
    /// Spawn a fresh drone: level 1 with the Zappy starting ration of 10 food.
    pub fn new(id: u32, team: usize, x: usize, y: usize, orientation: Orientation) -> Self {
        Player {
            id,
            team,
            x,
            y,
            orientation,
            level: 1,
            food: 10,
        }
    }
}
