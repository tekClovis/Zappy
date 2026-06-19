//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// command (AI action: parse + time cost + execute)
//

use super::map::Resource;
use super::player::Orientation;
use super::world::World;

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
            Command::Look => "[ ]".to_string(),
            Command::Inventory => world
                .players
                .get(&player)
                .map(|p| p.inventory_string())
                .unwrap_or_else(|| "ko".to_string()),
            Command::Take(item) => match Resource::from_name(item) {
                Some(res) if world.player_take(player, res) => "ok".to_string(),
                _ => "ko".to_string(),
            },
            Command::Set(item) => match Resource::from_name(item) {
                Some(res) if world.player_set(player, res) => "ok".to_string(),
                _ => "ko".to_string(),
            },
            Command::Incantation => "Elevation underway".to_string(),
            Command::ConnectNbr => "0".to_string(),
            Command::Broadcast(msg) => {
                for id in world.players.keys() {
                    if *id != player {
                        println!("[broadcast] #{player} -> #{id}: {msg}");
                    }
                }
                "ok".to_string()
            }
            _ => "ok".to_string(),
        }
    }
}
