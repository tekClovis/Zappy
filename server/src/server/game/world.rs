//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// world
//

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

use super::command::{Command, EnqueueError, MAX_QUEUED};
use super::gui::GuiEvent;
use super::map::{Map, Resource};
use super::player::{Orientation, Player, StarveResult};
use super::team::{Egg, JoinError, Team};

#[derive(Debug)]
pub enum Target {
    Player(u32),
    AllGui,
}

pub struct World {
    pub map: Map,
    pub teams: Vec<Team>,
    pub players: HashMap<u32, Player>,
    pub outbox: Vec<(Target, String)>,
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
            teams: names.iter().map(|name| Team::new(name.clone())).collect(),
            players: HashMap::new(),
            outbox: Vec::new(),
            next_id: 1,
            rng: seed,
        };
        for team_idx in 0..world.teams.len() {
            for _ in 0..clients {
                let (x, y) = world.random_tile();
                world.lay_egg(team_idx, x, y);
            }
        }
        world.respawn_resources();
        world
    }

    pub fn lay_egg(&mut self, team_idx: usize, x: usize, y: usize) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.teams[team_idx].lay_egg(id, x, y);
        id
    }

    pub fn remove_egg(&mut self, egg_id: u32) -> Option<Egg> {
        self.teams.iter_mut().find_map(|t| t.remove_egg(egg_id))
    }

    pub fn eggs_at(&self, x: usize, y: usize) -> Vec<u32> {
        self.teams.iter().flat_map(|t| t.eggs_at(x, y)).collect()
    }

    pub fn take_outbox(&mut self) -> Vec<(Target, String)> {
        std::mem::take(&mut self.outbox)
    }

    pub fn emit(&mut self, event: GuiEvent) {
        if let Some(line) = event.encode(self) {
            self.outbox.push((Target::AllGui, line));
        }
    }

    pub fn respawn_resources(&mut self) {
        let area = (self.map.width * self.map.height) as f64;
        let mut touched: HashSet<(usize, usize)> = HashSet::new();
        for res in Resource::ALL {
            let target = ((area * res.density()) as u32).max(1);
            for _ in self.map.total(res)..target {
                let (x, y) = self.random_tile();
                self.map.tile_mut(x, y).add(res, 1);
                touched.insert((x, y));
            }
        }
        for (x, y) in touched {
            self.emit(GuiEvent::Tile(x, y));
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
        let egg = self.teams[team_idx].hatch().ok_or(JoinError::TeamFull)?;

        let orientation = match self.next_rand() % 4 {
            0 => Orientation::North,
            1 => Orientation::East,
            2 => Orientation::South,
            _ => Orientation::West,
        };

        let id = self.next_id;
        self.next_id += 1;
        self.players
            .insert(id, Player::new(id, team_idx, egg.x, egg.y, orientation));
        self.teams[team_idx].players.push(id);
        self.emit(GuiEvent::EggHatched(egg.id));
        self.emit(GuiEvent::NewPlayer(id));
        Ok((id, self.teams[team_idx].remaining_eggs()))
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

    pub fn current_is_incantation(&self, player: u32) -> bool {
        self.players
            .get(&player)
            .and_then(|p| p.queue.front())
            .is_some_and(|c| *c == Command::Incantation)
    }

    pub fn abort_current(&mut self, player: u32) {
        if let Some(p) = self.players.get_mut(&player) {
            p.busy = false;
            p.current_cmd = 0;
            p.queue.pop_front();
        }
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
            self.emit(GuiEvent::PlayerDied(player));
        }
    }
}

/// Test helper: drain the outbox, keeping only the GUI-bound lines.
#[cfg(test)]
pub(crate) fn drain_gui(world: &mut World) -> Vec<String> {
    world
        .take_outbox()
        .into_iter()
        .filter_map(|(t, l)| matches!(t, Target::AllGui).then_some(l))
        .collect()
}

/// Test helper: owned team names from string literals.
#[cfg(test)]
pub(crate) fn names(n: &[&str]) -> Vec<String> {
    n.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_player_emits_ebo_then_pnw() {
        let mut w = World::new(3, 3, &names(&["t1"]), 1);
        let (id, _) = w.add_player("t1").expect("a free slot");
        let lines = drain_gui(&mut w);
        let ebo = lines.iter().position(|l| l.starts_with("ebo #")).unwrap();
        let pnw = lines
            .iter()
            .position(|l| l.starts_with(&format!("pnw #{id} ")))
            .unwrap();
        assert!(ebo < pnw, "ebo must precede pnw");
    }

    #[test]
    fn remove_player_emits_pdi() {
        let mut w = World::new(3, 3, &names(&["t1"]), 1);
        let (id, _) = w.add_player("t1").expect("a free slot");
        let _ = drain_gui(&mut w); // drain spawn events
        w.remove_player(id);
        assert!(drain_gui(&mut w).contains(&format!("pdi #{id}")));
    }

    #[test]
    fn respawn_emits_bct_for_touched_tiles_only() {
        let mut w = World::new(2, 2, &names(&["t1"]), 0);
        let _ = drain_gui(&mut w); // drain the startup respawn
        // Drain to a fresh slate, then deplete and respawn deterministically.
        for y in 0..2 {
            for x in 0..2 {
                for res in Resource::ALL {
                    let n = w.map.tile(x, y).count(res);
                    if n > 0 {
                        w.map.tile_mut(x, y).take(res, n);
                    }
                }
            }
        }
        w.respawn_resources();
        let lines = drain_gui(&mut w);
        assert!(!lines.is_empty());
        assert!(lines.iter().all(|l| l.starts_with("bct ")));
        // Never more than one bct per tile (2x2 = 4 tiles).
        assert!(lines.len() <= 4);
    }
}
