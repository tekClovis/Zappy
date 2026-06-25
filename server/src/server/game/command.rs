//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// command (AI action: parse + time cost + execute)
//

use std::collections::HashMap;

use super::gui::GuiEvent;
use super::map::{RESOURCE_COUNT, Resource, Tile};
use super::player::Orientation;
use super::world::{Target, World};

pub const MAX_QUEUED: usize = 10;

/// Players at level 8 a team needs to win the game (§7.5).
const WIN_PLAYERS: usize = 6;

/// Elevation prerequisites (§7.5), indexed by `level - 1` (level 1→2 .. 7→8).
/// Each entry is `(required players, [stones by Resource index])`; the food slot
/// (index 0) is always 0.
const ELEVATION: [(usize, [u32; RESOURCE_COUNT]); 7] = [
    (1, [0, 1, 0, 0, 0, 0, 0]),
    (2, [0, 1, 1, 1, 0, 0, 0]),
    (2, [0, 2, 0, 1, 0, 2, 0]),
    (4, [0, 1, 1, 2, 0, 1, 0]),
    (4, [0, 1, 2, 1, 3, 0, 0]),
    (6, [0, 1, 2, 3, 0, 1, 0]),
    (6, [0, 2, 2, 2, 2, 2, 1]),
];

/// Prerequisites to rise from `level`, or `None` once at the cap (level 8).
fn elevation(level: u8) -> Option<&'static (usize, [u32; RESOURCE_COUNT])> {
    if level == 0 || level as usize > ELEVATION.len() {
        None
    } else {
        Some(&ELEVATION[level as usize - 1])
    }
}

/// True when `tile` holds at least the `need` stones (by Resource index).
fn stones_present(tile: &Tile, need: &[u32; RESOURCE_COUNT]) -> bool {
    need.iter()
        .enumerate()
        .all(|(res, &n)| tile.count(Resource::ALL[res]) >= n)
}

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
                world.emit(GuiEvent::Moved(player));
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
                world.emit(GuiEvent::Moved(player));
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
                world.emit(GuiEvent::Moved(player));
                "ok".to_string()
            }
            Command::Look => {
                let (px, py, orientation, level) = match world.players.get(&player) {
                    Some(p) => (p.x, p.y, p.orientation, p.level),
                    None => return "ko".to_string(),
                };
                let (fx, fy) = orientation.to_vec();
                let (rx, ry) = (-fy, fx);
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
                let Some(res) = Resource::from_name(item) else {
                    return "ko".to_string();
                };
                let Some((x, y)) = world.players.get(&player).map(|p| (p.x, p.y)) else {
                    return "ko".to_string();
                };
                if !world.map.tile_mut(x, y).take_one(res) {
                    return "ko".to_string();
                }
                if let Some(p) = world.players.get_mut(&player) {
                    p.inventory[res as usize] += 1;
                }
                world.emit(GuiEvent::Collected {
                    player,
                    res: res as usize,
                });
                world.emit(GuiEvent::Inventory(player));
                world.emit(GuiEvent::Tile(x, y));
                "ok".to_string()
            }
            Command::Set(item) => {
                let Some(res) = Resource::from_name(item) else {
                    return "ko".to_string();
                };
                let Some((x, y)) = world.players.get(&player).map(|p| (p.x, p.y)) else {
                    return "ko".to_string();
                };
                {
                    let Some(p) = world.players.get_mut(&player) else {
                        return "ko".to_string();
                    };
                    if p.inventory[res as usize] == 0 {
                        return "ko".to_string();
                    }
                    p.inventory[res as usize] -= 1;
                }
                world.map.tile_mut(x, y).add(res, 1);
                world.emit(GuiEvent::Dropped {
                    player,
                    res: res as usize,
                });
                world.emit(GuiEvent::Inventory(player));
                world.emit(GuiEvent::Tile(x, y));
                "ok".to_string()
            }
            Command::Fork => {
                let (px, py, team) = match world.players.get(&player) {
                    Some(p) => (p.x, p.y, p.team),
                    None => return "ok".to_string(),
                };
                let egg_id = world.lay_egg(team, px, py);
                world.emit(GuiEvent::Forked(player));
                world.emit(GuiEvent::EggLaid {
                    egg: egg_id,
                    player,
                    x: px,
                    y: py,
                });
                "ok".to_string()
            }
            Command::Incantation => "Elevation underway".to_string(),
            Command::ConnectNbr => world
                .players
                .get(&player)
                .and_then(|p| world.teams.get(p.team))
                .map(|t| t.remaining_eggs())
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
                world.emit(GuiEvent::Broadcast {
                    player,
                    msg: msg.clone(),
                });
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

                let ejected = !pushed.is_empty();
                for (id, k) in pushed {
                    if let Some(o) = world.players.get_mut(&id) {
                        o.x = new_x;
                        o.y = new_y;
                    }
                    world
                        .outbox
                        .push((Target::Player(id), format!("eject: {k}")));
                    world.emit(GuiEvent::Moved(id));
                }
                if ejected {
                    world.emit(GuiEvent::Ejected(player));
                }
                for egg_id in world.eggs_at(px, py) {
                    if world.remove_egg(egg_id).is_some() {
                        world.emit(GuiEvent::EggDestroyed(egg_id));
                    }
                }
                "ok".to_string()
            }
        }
    }

    pub fn incantation_start(
        world: &mut World,
        player: u32,
    ) -> Option<(u8, (usize, usize), Vec<u32>)> {
        let (px, py, level) = world.players.get(&player).map(|p| (p.x, p.y, p.level))?;
        let (need_players, need_stones) = *elevation(level)?;

        // Every same-level player on the tile takes part — the initiator plus the
        // others — counted by position+level, not idleness, matching the reference's
        // `get_incantating_player`. The headcount gates the requirement.
        let mut participants = vec![player];
        participants.extend(
            world
                .players
                .values()
                .filter(|o| o.id != player && o.x == px && o.y == py && o.level == level)
                .map(|o| o.id),
        );
        if participants.len() < need_players {
            return None;
        }
        if !stones_present(world.map.tile(px, py), &need_stones) {
            return None;
        }

        // Freeze every participant for the ritual. One caught mid-command has its
        // in-flight action invalidated (`current_cmd = 0`) so the pending `ActionDone`
        // becomes a no-op and the queued command re-runs once unfrozen — this keeps it
        // from walking off the tile and failing the end-of-cast headcount (the level-6
        // ceiling).
        for &id in &participants[1..] {
            if let Some(o) = world.players.get_mut(&id) {
                o.busy = true;
                o.current_cmd = 0;
            }
        }
        world
            .outbox
            .push((Target::Player(player), "Elevation underway".to_string()));
        world.emit(GuiEvent::IncantationStart {
            x: px,
            y: py,
            level,
            participants: participants.clone(),
        });
        Some((level, (px, py), participants))
    }

    #[cfg(test)]
    fn run(&self, world: &mut World, player: u32) -> (String, Vec<String>) {
        let reply = self.execute(world, player);
        (reply, super::world::drain_gui(world))
    }

    pub fn incantation_finish(
        world: &mut World,
        tile: (usize, usize),
        level: u8,
        participants: &[u32],
    ) {
        let (px, py) = tile;
        if let Some(&actor) = participants.first() {
            world.abort_current(actor);
        }
        for &id in &participants[1..] {
            if let Some(o) = world.players.get_mut(&id) {
                o.busy = false;
            }
        }

        let present: Vec<u32> = world
            .players
            .values()
            .filter(|p| p.level == level && p.x == px && p.y == py)
            .map(|p| p.id)
            .collect();

        let Some(&(need_players, need_stones)) = elevation(level) else {
            return;
        };
        if present.len() < need_players || !stones_present(world.map.tile(px, py), &need_stones) {
            if let Some(&actor) = participants.first() {
                world.outbox.push((Target::Player(actor), "ko".to_string()));
            }
            world.emit(GuiEvent::IncantationEnd {
                x: px,
                y: py,
                result: 0,
            });
            return;
        }

        let cell = world.map.tile_mut(px, py);
        for (res, &n) in need_stones.iter().enumerate() {
            cell.take(Resource::ALL[res], n);
        }

        let new_level = level + 1;
        for &id in &present {
            if let Some(p) = world.players.get_mut(&id) {
                p.level = new_level;
            }
            world
                .outbox
                .push((Target::Player(id), format!("Current level: {new_level}")));
            world.emit(GuiEvent::LevelChanged(id));
        }
        world.emit(GuiEvent::IncantationEnd {
            x: px,
            y: py,
            result: 1,
        });
        world.emit(GuiEvent::Tile(px, py));

        let winners: Vec<String> = world
            .teams
            .iter()
            .filter(|t| {
                t.players
                    .iter()
                    .filter(|id| world.players.get(id).is_some_and(|p| p.level >= 8))
                    .count()
                    >= WIN_PLAYERS
            })
            .map(|t| t.name.clone())
            .collect();
        for name in winners {
            world.emit(GuiEvent::GameWon(name));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::game::world::names;

    fn world_with_player() -> (World, u32) {
        let mut w = World::new(5, 5, &names(&["t1"]), 1);
        let (id, _) = w.add_player("t1").expect("a free slot");
        let _ = w.take_outbox(); // drain spawn events
        (w, id)
    }

    #[test]
    fn forward_emits_ppo() {
        let (mut w, id) = world_with_player();
        let (reply, gui) = Command::Forward.run(&mut w, id);
        assert_eq!(reply, "ok");
        assert_eq!(gui.len(), 1);
        assert!(gui[0].starts_with(&format!("ppo #{id} ")));
    }

    #[test]
    fn take_success_emits_pgt_pin_bct() {
        let (mut w, id) = world_with_player();
        let (x, y) = w.players.get(&id).map(|p| (p.x, p.y)).unwrap();
        w.map.tile_mut(x, y).add(Resource::Linemate, 1);
        let (reply, gui) = Command::Take("linemate".to_string()).run(&mut w, id);
        assert_eq!(reply, "ok");
        let li = Resource::Linemate as usize;
        assert!(gui.contains(&format!("pgt #{id} {li}")));
        assert!(gui.iter().any(|l| l.starts_with(&format!("pin #{id} "))));
        assert!(gui.iter().any(|l| l.starts_with(&format!("bct {x} {y} "))));
    }

    #[test]
    fn take_missing_resource_is_ko_and_silent() {
        let (mut w, id) = world_with_player();
        let (x, y) = w.players.get(&id).map(|p| (p.x, p.y)).unwrap();
        for res in Resource::ALL {
            let n = w.map.tile(x, y).count(res);
            if n > 0 {
                w.map.tile_mut(x, y).take(res, n);
            }
        }
        let (reply, gui) = Command::Take("linemate".to_string()).run(&mut w, id);
        assert_eq!(reply, "ko");
        assert!(gui.is_empty());
    }

    #[test]
    fn set_success_emits_pdr_pin_bct() {
        let (mut w, id) = world_with_player();
        let (x, y) = w.players.get(&id).map(|p| (p.x, p.y)).unwrap();
        // Fresh player has food; drop one.
        let (reply, gui) = Command::Set("food".to_string()).run(&mut w, id);
        assert_eq!(reply, "ok");
        let fi = Resource::Food as usize;
        assert!(gui.contains(&format!("pdr #{id} {fi}")));
        assert!(gui.iter().any(|l| l.starts_with(&format!("pin #{id} "))));
        assert!(gui.iter().any(|l| l.starts_with(&format!("bct {x} {y} "))));
    }

    #[test]
    fn broadcast_emits_pbc() {
        let (mut w, id) = world_with_player();
        let (reply, gui) = Command::Broadcast("hello world".to_string()).run(&mut w, id);
        assert_eq!(reply, "ok");
        assert!(gui.contains(&format!("pbc #{id} hello world")));
    }

    /// Regression: a 6->7 incantation needs six players. It must elevate all of
    /// them even when some are mid-command — those participants are frozen and
    /// their in-flight action is invalidated so they cannot walk off the tile and
    /// fail the end-of-cast headcount (the old level-6 ceiling).
    #[test]
    fn incantation_elevates_six_including_busy_players() {
        let mut w = World::new(5, 5, &names(&["t1"]), 6);
        let ids: Vec<u32> = (0..6)
            .map(|_| w.add_player("t1").expect("a free egg").0)
            .collect();
        let _ = w.take_outbox(); // drain spawn events

        // Gather all six on one tile at level 6.
        for &id in &ids {
            let p = w.players.get_mut(&id).unwrap();
            (p.x, p.y, p.level) = (2, 2, 6);
        }
        // Stones for 6 -> 7: linemate 1, deraumere 2, sibur 3, phiras 1.
        let tile = w.map.tile_mut(2, 2);
        tile.add(Resource::Linemate, 1);
        tile.add(Resource::Deraumere, 2);
        tile.add(Resource::Sibur, 3);
        tile.add(Resource::Phiras, 1);

        // Two non-initiators are mid-command (each just started a Forward).
        for &id in &ids[1..3] {
            assert!(w.enqueue_command(id, "Forward").is_ok());
            w.start_next(id).expect("starts the queued Forward");
            assert!(w.players[&id].busy && w.players[&id].current_cmd != 0);
        }

        // Initiator casts.
        let actor = ids[0];
        assert!(w.enqueue_command(actor, "Incantation").is_ok());
        w.start_next(actor).expect("starts the incantation");
        let (level, xy, participants) =
            Command::incantation_start(&mut w, actor).expect("requirements met");
        assert_eq!(level, 6);
        assert_eq!(participants.len(), 6, "all six on the tile participate");
        // Every other participant is frozen, busy ones with their action voided.
        for &id in &ids[1..] {
            assert!(w.players[&id].busy, "participant frozen");
            assert_eq!(
                w.players[&id].current_cmd, 0,
                "in-flight action invalidated"
            );
        }

        Command::incantation_finish(&mut w, xy, level, &participants);
        for &id in &ids {
            assert_eq!(w.players[&id].level, 7, "player #{id} reached level 7");
        }
        // The interrupted Forward survived to re-run and never moved its caster.
        assert_eq!(w.players[&ids[1]].queue.front(), Some(&Command::Forward));
        assert_eq!((w.players[&ids[1]].x, w.players[&ids[1]].y), (2, 2));
    }
}
