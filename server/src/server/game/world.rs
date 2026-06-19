//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// world
//

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::command::{Command, EnqueueError, MAX_QUEUED};
use super::map::{Map, Resource};
use super::player::{Orientation, Player, StarveResult};
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
        let mut world = World {
            map: Map::new(width, height),
            teams: names
                .iter()
                .map(|name| Team::new(name.clone(), clients))
                .collect(),
            players: HashMap::new(),
            next_id: 1,
            rng: seed,
        };
        world.respawn_resources();
        world
    }

    pub fn respawn_resources(&mut self) {
        let area = (self.map.width * self.map.height) as f64;
        for res in Resource::ALL {
            let target = ((area * res.density()) as u32).max(1);
            for _ in self.map.total(res)..target {
                let (x, y) = self.random_tile();
                self.map.tile_mut(x, y).add(res, 1);
            }
        }
    }

    fn next_rand(&mut self) -> u64 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        self.rng
    }

    fn random_tile(&mut self) -> (usize, usize) {
        let rx = self.next_rand() as isize;
        let ry = self.next_rand() as isize;
        self.map.wrap(rx, ry)
    }

    pub fn add_player(&mut self, team_name: &str) -> Result<(u32, usize), JoinError> {
        let team_idx = self
            .teams
            .iter()
            .position(|t| t.name == team_name)
            .ok_or(JoinError::UnknownTeam)?;
        if !self.teams[team_idx].has_free_slot() {
            return Err(JoinError::TeamFull);
        }

        let (x, y) = self.random_tile();
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

    pub fn consume_food(&mut self, player: u32) -> StarveResult {
        match self.players.get_mut(&player) {
            None => StarveResult::Gone,
            Some(p) => {
                let food = &mut p.inventory[Resource::Food as usize];
                *food = food.saturating_sub(1);
                if *food == 0 {
                    StarveResult::Died
                } else {
                    StarveResult::Survived
                }
            }
        }
    }

    pub fn remove_player(&mut self, player: u32) {
        if let Some(p) = self.players.remove(&player) {
            if let Some(team) = self.teams.get_mut(p.team) {
                team.players.retain(|&id| id != player);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starvation_drains_food_then_dies() {
        let names = vec!["team1".to_string()];
        let mut world = World::new(10, 10, &names, 5);
        let (id, _) = world.add_player("team1").expect("join");

        // 10 starting food: first 9 ticks survive, the 10th empties it and dies.
        for _ in 0..9 {
            assert_eq!(world.consume_food(id), StarveResult::Survived);
        }
        assert_eq!(world.consume_food(id), StarveResult::Died);
    }
}
