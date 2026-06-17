//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// command (AI action: parse + time cost + execute)
//

use super::world::World;

/// Per-player ceiling on queued commands (ARCHITECTURE.md §3): an AI may stack up
/// to 10 actions; anything beyond is dropped.
pub const MAX_QUEUED: usize = 10;

/// One parsed AI action. `Broadcast`/`Take`/`Set` carry their text argument.
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

/// Why a line could not be accepted into a player's queue.
pub enum EnqueueError {
    /// Unknown verb or a missing argument: reply `ko` to the AI.
    BadCommand,
    /// Queue already holds `MAX_QUEUED`: silently dropped.
    QueueFull,
}

impl Command {
    /// Parse one protocol line into a `Command`. Returns `None` for an unknown
    /// verb or a missing argument (`Take`/`Set`/`Broadcast` need a non-empty one).
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

    /// Cost in time units; the real delay is `cost / f` seconds (ARCHITECTURE.md §5).
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

    /// Placeholder effect: the real world mutation + GUI events land later. For
    /// now, print what would run and return the AI's canned wire reply.
    pub fn execute(&self, world: &mut World, player: u32) -> String {
        println!("[cmd] #{player} {self:?}");
        let _ = world;
        // notify_gui(...) — emit ppo/pgt/pic/... here once the GUI sink exists.
        match self {
            Command::Look => "[ ]".to_string(),
            Command::Inventory => "[ ]".to_string(),
            Command::ConnectNbr => "0".to_string(),
            Command::Incantation => "Elevation underway".to_string(),
            _ => "ok".to_string(),
        }
    }
}
