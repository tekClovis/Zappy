//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// command (AI action: parse + time cost + execute)
//

use std::collections::HashMap;

use super::map::Resource;
use super::player::Orientation;
use super::world::{Target, World};

pub const MAX_QUEUED: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Forward,
    Right,
    Left,
    Look,
    Inventory,
    Broadcast(String),
    ConnectNbr,
    Fork,
    Eject,
    Take(String),
    Set(String),
    Incantation,
}

pub enum EnqueueError {
    /// Unknown verb or a missing argument: reply `ko` to the AI.
    BadCommand,
    /// Queue already holds `MAX_QUEUED`: silently dropped.
    QueueFull,
}

impl Command {
    pub fn parse(line: &str) -> Option<Command> {
        let (verb, rest) = match line.trim().split_once(' ') {
            Some((v, r)) => (v, r.trim()),
            None => (line.trim(), ""),
        };
        let with_arg = |c: fn(String) -> Command| {
            if rest.is_empty() {
                None
            } else {
                Some(c(rest.to_string()))
            }
        };
        match verb {
            "Forward" => Some(Command::Forward),
            "Right" => Some(Command::Right),
            "Left" => Some(Command::Left),
            "Look" => Some(Command::Look),
            "Inventory" => Some(Command::Inventory),
            "Connect_nbr" => Some(Command::ConnectNbr),
            "Fork" => Some(Command::Fork),
            "Eject" => Some(Command::Eject),
            "Incantation" => Some(Command::Incantation),
            "Broadcast" => with_arg(Command::Broadcast),
            "Take" => with_arg(Command::Take),
            "Set" => with_arg(Command::Set),
            _ => None,
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            Command::ConnectNbr => 0,
            Command::Inventory => 1,
            Command::Fork => 42,
            Command::Incantation => 300,
            Command::Forward
            | Command::Right
            | Command::Left
            | Command::Look
            | Command::Broadcast(_)
            | Command::Eject
            | Command::Take(_)
            | Command::Set(_) => 7,
        }
    }

    pub fn execute(&self, world: &mut World, player: u32) -> String {
        println!("[cmd] #{player} {self:?}");
        match self {
            Command::Forward => {
                if let Some(p) = world.players.get_mut(&player) {
                    let (dx, dy) = match p.orientation {
                        Orientation::North => (0, -1),
                        Orientation::South => (0, 1),
                        Orientation::East => (1, 0),
                        Orientation::West => (-1, 0),
                    };
                    let (new_x, new_y) = world.map.wrap(p.x as isize + dx, p.y as isize + dy);
                    p.x = new_x;
                    p.y = new_y;
                }
                "ok".to_string()
            }
            Command::Right => {
                if let Some(p) = world.players.get_mut(&player) {
                    p.orientation = match p.orientation {
                        Orientation::North => Orientation::East,
                        Orientation::East => Orientation::South,
                        Orientation::South => Orientation::West,
                        Orientation::West => Orientation::North,
                    };
                }
                "ok".to_string()
            }
            Command::Left => {
                if let Some(p) = world.players.get_mut(&player) {
                    p.orientation = match p.orientation {
                        Orientation::North => Orientation::West,
                        Orientation::West => Orientation::South,
                        Orientation::South => Orientation::East,
                        Orientation::East => Orientation::North,
                    };
                }
                "ok".to_string()
            }
            Command::Look => {
                let (px, py, orientation, level) = match world.players.get(&player) {
                    Some(p) => (p.x, p.y, p.orientation, p.level),
                    None => return "ko".to_string(),
                };
                let (fx, fy) = orientation.to_vec();
                // `right` is `forward` rotated 90° clockwise; rows run left→right.
                let (rx, ry) = (-fy, fx);
                // One pass over players so each tile is an O(1) lookup, not a re-scan.
                let mut drones: HashMap<(usize, usize), usize> = HashMap::new();
                for o in world.players.values() {
                    *drones.entry((o.x, o.y)).or_insert(0) += 1;
                }
                let mut tiles: Vec<String> = Vec::new();
                for d in 0..=level as isize {
                    for lat in -d..=d {
                        let (tx, ty) = world.map.wrap(
                            px as isize + fx * d + rx * lat,
                            py as isize + fy * d + ry * lat,
                        );
                        let tile = world.map.tile(tx, ty);
                        let count = drones.get(&(tx, ty)).copied().unwrap_or(0);
                        let tokens = std::iter::repeat_n("player", count).chain(
                            Resource::ALL.iter().flat_map(|&res| {
                                std::iter::repeat_n(res.name(), tile.count(res) as usize)
                            }),
                        );
                        tiles.push(tokens.collect::<Vec<_>>().join(" "));
                    }
                }
                format!("[{}]", tiles.join(", "))
            }
            Command::Inventory => world
                .players
                .get(&player)
                .map(|p| p.inventory_string())
                .unwrap_or_else(|| "ko".to_string()),
            Command::Take(item) => {
                if let Some(res) = Resource::from_name(item) {
                    if let Some(p) = world.players.get_mut(&player) {
                        if world.map.tile_mut(p.x, p.y).take_one(res) {
                            p.inventory[res as usize] += 1;
                            return "ok".to_string();
                        }
                    }
                }
                "ko".to_string()
            }
            Command::Set(item) => {
                if let Some(res) = Resource::from_name(item) {
                    if let Some(p) = world.players.get_mut(&player) {
                        if p.inventory[res as usize] > 0 {
                            p.inventory[res as usize] -= 1;
                            world.map.tile_mut(p.x, p.y).add(res, 1);
                            return "ok".to_string();
                        }
                    }
                }
                "ko".to_string()
            }
            Command::Incantation => "Elevation underway".to_string(),
            Command::ConnectNbr => world
                .players
                .get(&player)
                .and_then(|p| world.teams.get(p.team))
                .map(|t| t.remaining())
                .unwrap_or(0)
                .to_string(),
            Command::Broadcast(msg) => {
                let (ex, ey) = match world.players.get(&player) {
                    Some(p) => (p.x, p.y),
                    None => return "ok".to_string(),
                };

                let recipients: Vec<(u32, u8)> = world
                    .players
                    .values()
                    .filter(|o| o.id != player)
                    .map(|o| {
                        let offset = world.map.shortest_offset((o.x, o.y), (ex, ey));
                        (o.id, o.orientation.sound_dir(offset))
                    })
                    .collect();
                for (id, k) in recipients {
                    world
                        .outbox
                        .push((Target::Player(id), format!("message {k}, {msg}")));
                }
                "ok".to_string()
            }
            Command::Eject => {
                let (px, py, orientation) = match world.players.get(&player) {
                    Some(p) => (p.x, p.y, p.orientation),
                    None => return "ok".to_string(),
                };
                let (dx, dy) = orientation.to_vec();
                let (new_x, new_y) = world.map.wrap(px as isize + dx, py as isize + dy);

                let pushed: Vec<(u32, u8)> = world
                    .players
                    .values()
                    .filter(|o| o.id != player && o.x == px && o.y == py)
                    .map(|o| (o.id, o.orientation.sound_dir((-dx, -dy))))
                    .collect();

                for (id, k) in pushed {
                    if let Some(o) = world.players.get_mut(&id) {
                        o.x = new_x;
                        o.y = new_y;
                    }
                    world
                        .outbox
                        .push((Target::Player(id), format!("eject: {k}")));
                }
                "ok".to_string()
            }
            _ => "ok".to_string(),
        }
    }
}
