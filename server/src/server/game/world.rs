//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// world
//

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::player::{Orientation, Player};
use super::team::{JoinError, Team};

pub struct World {
    pub w_h: (usize, usize),
    pub teams: Vec<Team>,
    pub players: HashMap<u32, Player>,
    next_id: u32,
    rng: u64,
}

impl World {
    pub fn new(width: usize, height: usize, names: &[String], clients: usize) -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9e37_79b9_7f4a_7c15)
            | 1;
        World {
            w_h: (width, height),
            teams: names
                .iter()
                .map(|name| Team::new(name.clone(), clients))
                .collect(),
            players: HashMap::new(),
            next_id: 1,
            rng: seed,
        }
    }

    /// xorshift64: a small std-only PRNG, just enough to scatter spawns.
    fn next_rand(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    /// Hatch a drone for `team_name`: assign an id, a random tile + facing,
    /// consume one slot, and store the player.
    ///
    /// Returns `(player_id, remaining_slots)` on success.
    pub fn add_player(&mut self, team_name: &str) -> Result<(u32, usize), JoinError> {
        let team_idx = self
            .teams
            .iter()
            .position(|t| t.name == team_name)
            .ok_or(JoinError::UnknownTeam)?;
        if !self.teams[team_idx].has_free_slot() {
            return Err(JoinError::TeamFull);
        }

        let (width, height) = self.w_h;
        let x = (self.next_rand() as usize) % width;
        let y = (self.next_rand() as usize) % height;
        let orientation = match self.next_rand() % 4 {
            0 => Orientation::North,
            1 => Orientation::East,
            2 => Orientation::South,
            _ => Orientation::West,
        };

        let id = self.next_id;
        self.next_id += 1;
        self.players
            .insert(id, Player::new(id, team_idx, x, y, orientation));

        let team = &mut self.teams[team_idx];
        team.slots -= 1;
        team.players.push(id);
        Ok((id, team.remaining()))
    }
}
