//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// world
//

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::command::{Command, EnqueueError, MAX_QUEUED};
use super::map::Map;
use super::player::{Orientation, Player};
use super::team::{JoinError, Team};

pub struct World {
    pub map: Map,
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
            map: Map::new(width, height),
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

        let rx = self.next_rand() as isize;
        let ry = self.next_rand() as isize;
        let (x, y) = self.map.wrap(rx, ry);
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

    /// Parse `line` and push it onto `player`'s bounded queue. Reply handling
    /// (the `ko` on `BadCommand`, the silent drop on `QueueFull`) is the reactor's.
    pub fn enqueue_command(&mut self, player: u32, line: &str) -> Result<(), EnqueueError> {
        let cmd = Command::parse(line).ok_or(EnqueueError::BadCommand)?;
        let p = self
            .players
            .get_mut(&player)
            .ok_or(EnqueueError::BadCommand)?;
        if p.queue.len() >= MAX_QUEUED {
            return Err(EnqueueError::QueueFull);
        }
        p.queue.push_back(cmd);
        Ok(())
    }

    /// If `player` is idle with a queued command, mark it in-flight and return
    /// `(command_id, cost)` for the reactor to schedule. `None` = nothing to
    /// start or the player is already busy.
    pub fn start_next(&mut self, player: u32) -> Option<(u64, u32)> {
        let p = self.players.get_mut(&player)?;
        if p.busy {
            return None;
        }
        let cost = p.queue.front()?.cost();
        p.cmd_seq += 1;
        p.current_cmd = p.cmd_seq;
        p.busy = true;
        Some((p.current_cmd, cost))
    }

    /// An `ActionDone` fired: if it matches the in-flight command, pop and
    /// execute it (placeholder), returning the AI reply. Clears `busy` so the
    /// next queued command can start.
    pub fn finish_command(&mut self, player: u32, command_id: u64) -> Option<String> {
        let cmd = {
            let p = self.players.get_mut(&player)?;
            if !p.busy || p.current_cmd != command_id {
                return None;
            }
            p.busy = false;
            p.current_cmd = 0;
            p.queue.pop_front()?
        };
        Some(cmd.execute(self, player))
    }

    /// Drop a drone when its AI disconnects or starves (ref GUI protocol: `pdi`).
    /// Removes it from the world and its team roster. The team **slot is not
    /// returned** — a slot only comes back when a `Fork` lays a new egg.
    pub fn remove_player(&mut self, player: u32) {
        if let Some(p) = self.players.remove(&player) {
            if let Some(team) = self.teams.get_mut(p.team) {
                team.players.retain(|&id| id != player);
            }
        }
    }
}
