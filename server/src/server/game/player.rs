//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// player (drone: pos, orientation, level, inventory)
//

use super::command::Command;
use super::map::{RESOURCE_COUNT, Resource};
use std::collections::VecDeque;
use std::fmt;

pub const STARVE_INTERVAL_UNITS: u32 = 126;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StarveResult {
    Survived,
    Died,
    Gone,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundDir {
    North = 1,
    NorthWest = 2,
    West = 3,
    SouthWest = 4,
    South = 5,
    SouthEast = 6,
    East = 7,
    NorthEast = 8,
}

impl fmt::Display for SoundDir {
    /// Emits the subject's direction number `K` (1..=8), e.g. for `eject: K`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

impl SoundDir {
    #[allow(dead_code)]
    pub fn to_orientation(self) -> Option<Orientation> {
        match self {
            SoundDir::North => Some(Orientation::North),
            SoundDir::East => Some(Orientation::East),
            SoundDir::South => Some(Orientation::South),
            SoundDir::West => Some(Orientation::West),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    North = 1,
    East = 2,
    South = 3,
    West = 4,
}

impl Orientation {
    #[allow(dead_code)]
    pub fn to_song(self) -> SoundDir {
        match self {
            Orientation::North => SoundDir::North,
            Orientation::East => SoundDir::East,
            Orientation::South => SoundDir::South,
            Orientation::West => SoundDir::West,
        }
    }

    pub fn sound_dir(self, offset: (isize, isize)) -> u8 {
        let (dx, dy) = offset;
        if dx == 0 && dy == 0 {
            return 0;
        }
        let (fx, fy) = self.to_vec();
        let forward = (dx * fx + dy * fy) as f64; // +ahead
        let left = (dx * fy - dy * fx) as f64; // +left (= right rotated CCW)
        let mut angle = left.atan2(forward).to_degrees();
        if angle < 0.0 {
            angle += 360.0;
        }
        ((angle / 45.0).round() as i64).rem_euclid(8) as u8 + 1
    }

    #[allow(dead_code)]
    pub fn from_vec(u: (isize, isize)) -> Option<Orientation> {
        match u {
            (0, -1) => Some(Orientation::North),
            (1, 0) => Some(Orientation::East),
            (0, 1) => Some(Orientation::South),
            (-1, 0) => Some(Orientation::West),
            _ => None,
        }
    }

    pub fn to_vec(self) -> (isize, isize) {
        match self {
            Orientation::North => (0, -1),
            Orientation::East => (1, 0),
            Orientation::South => (0, 1),
            Orientation::West => (-1, 0),
        }
    }
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

    #[test]
    fn orientation_song_roundtrip() {
        for o in [
            Orientation::North,
            Orientation::East,
            Orientation::South,
            Orientation::West,
        ] {
            assert_eq!(o.to_song().to_orientation(), Some(o));
        }
        assert_eq!(SoundDir::NorthWest.to_orientation(), None);
        assert_eq!(Orientation::East.to_song() as u8, 7); // PDF trig numbering
    }

    #[test]
    fn sound_dir_same_tile_is_zero() {
        assert_eq!(Orientation::North.sound_dir((0, 0)), 0);
        assert_eq!(Orientation::East.sound_dir((0, 0)), 0);
    }

    #[test]
    fn sound_dir_eight_neighbours_facing_north() {
        // (dx, dy) world offset (x East, y South) -> K, receiver facing North.
        let cases = [
            ((0, -1), 1u8), // ahead
            ((-1, -1), 2),  // front-left
            ((-1, 0), 3),   // left
            ((-1, 1), 4),   // back-left
            ((0, 1), 5),    // behind
            ((1, 1), 6),    // back-right
            ((1, 0), 7),    // right
            ((1, -1), 8),   // front-right
        ];
        for (offset, k) in cases {
            assert_eq!(Orientation::North.sound_dir(offset), k, "offset {offset:?}");
        }
    }

    #[test]
    fn sound_dir_rotates_with_facing() {
        // A source directly East in the world is "ahead" for an East-facing
        // drone and "behind" for a West-facing one.
        assert_eq!(Orientation::East.sound_dir((1, 0)), 1);
        assert_eq!(Orientation::West.sound_dir((1, 0)), 5);
        // Far-field still quantises to the nearest of the 8 tiles.
        assert_eq!(Orientation::North.sound_dir((5, -1)), 7);
    }
}
