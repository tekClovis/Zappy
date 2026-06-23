//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// gui (server <-> GUI protocol: event encoder + request parser)
//

use std::fmt::Write;

use super::map::RESOURCE_COUNT;
use super::world::World;

fn quantities(slots: &[u32; RESOURCE_COUNT]) -> String {
    let mut out = String::new();
    for (i, n) in slots.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let _ = write!(out, "{n}");
    }
    out
}

#[derive(Debug)]
pub enum GuiEvent {
    /// `msz X Y`
    MapSize,
    /// `tna N` for one team
    TeamName(usize),
    /// `bct X Y q0..q6` for one tile
    Tile(usize, usize),
    /// `pnw #n X Y O L N`
    NewPlayer(u32),
    /// `ppo #n X Y O`
    Moved(u32),
    /// `plv #n L`
    LevelChanged(u32),
    /// `pin #n X Y q0..q6`
    Inventory(u32),
    /// `pgt #n i`
    Collected { player: u32, res: usize },
    /// `pdr #n i`
    Dropped { player: u32, res: usize },
    /// `pbc #n M`
    Broadcast { player: u32, msg: String },
    /// `pex #n`
    Ejected(u32),
    /// `pfk #n`
    Forked(u32),
    /// `enw #e #n X Y`
    EggLaid {
        egg: u32,
        player: u32,
        x: usize,
        y: usize,
    },
    /// `ebo #e`
    EggHatched(u32),
    /// `edi #e`
    EggDestroyed(u32),
    /// `pdi #n`
    PlayerDied(u32),
    /// `pic X Y L #n #n ...`
    IncantationStart {
        x: usize,
        y: usize,
        level: u8,
        participants: Vec<u32>,
    },
    /// `pie X Y R`
    IncantationEnd { x: usize, y: usize, result: u8 },
    /// `seg N`
    GameWon(String),
}

impl GuiEvent {
    pub fn encode(&self, world: &World) -> Option<String> {
        Some(match self {
            GuiEvent::MapSize => format!("msz {} {}", world.map.width, world.map.height),
            GuiEvent::TeamName(idx) => format!("tna {}", world.teams.get(*idx)?.name),
            GuiEvent::Tile(x, y) => {
                if *x >= world.map.width || *y >= world.map.height {
                    return None;
                }
                format!(
                    "bct {x} {y} {}",
                    quantities(&world.map.tile(*x, *y).resources)
                )
            }
            GuiEvent::NewPlayer(id) => {
                let p = world.players.get(id)?;
                let name = &world.teams.get(p.team)?.name;
                format!(
                    "pnw #{id} {} {} {} {} {name}",
                    p.x, p.y, p.orientation as u8, p.level
                )
            }
            GuiEvent::Moved(id) => {
                let p = world.players.get(id)?;
                format!("ppo #{id} {} {} {}", p.x, p.y, p.orientation as u8)
            }
            GuiEvent::LevelChanged(id) => {
                let p = world.players.get(id)?;
                format!("plv #{id} {}", p.level)
            }
            GuiEvent::Inventory(id) => {
                let p = world.players.get(id)?;
                format!("pin #{id} {} {} {}", p.x, p.y, quantities(&p.inventory))
            }
            GuiEvent::Collected { player, res } => format!("pgt #{player} {res}"),
            GuiEvent::Dropped { player, res } => format!("pdr #{player} {res}"),
            GuiEvent::Broadcast { player, msg } => format!("pbc #{player} {msg}"),
            GuiEvent::Ejected(id) => format!("pex #{id}"),
            GuiEvent::Forked(id) => format!("pfk #{id}"),
            GuiEvent::EggLaid { egg, player, x, y } => format!("enw #{egg} #{player} {x} {y}"),
            GuiEvent::EggHatched(egg) => format!("ebo #{egg}"),
            GuiEvent::EggDestroyed(egg) => format!("edi #{egg}"),
            GuiEvent::PlayerDied(id) => format!("pdi #{id}"),
            GuiEvent::IncantationStart {
                x,
                y,
                level,
                participants,
            } => {
                let ids = participants
                    .iter()
                    .map(|id| format!("#{id}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("pic {x} {y} {level} {ids}")
            }
            GuiEvent::IncantationEnd { x, y, result } => format!("pie {x} {y} {result}"),
            GuiEvent::GameWon(name) => format!("seg {name}"),
        })
    }
}

/// A GUI -> server request line (the inbound half of the protocol).
#[derive(Debug)]
pub enum GuiRequest {
    /// `msz`
    MapSize,
    /// `bct X Y`
    Tile(usize, usize),
    /// `mct`
    MapContent,
    /// `tna`
    TeamNames,
    /// `ppo #n`
    PlayerPos(u32),
    /// `plv #n`
    PlayerLevel(u32),
    /// `pin #n`
    PlayerInv(u32),
    /// `sgt`
    TimeGet,
    /// `sst T`
    TimeSet(u32),
}

pub enum GuiParse {
    /// Unknown verb → reply `suc`.
    Unknown,
    /// Known verb with a malformed argument → reply `sbp`.
    BadParam,
}

impl GuiRequest {
    pub fn parse(line: &str) -> Result<GuiRequest, GuiParse> {
        let mut parts = line.split_whitespace();
        let verb = parts.next().unwrap_or("");
        let id = |s: Option<&str>| -> Result<u32, GuiParse> {
            s.and_then(|s| s.strip_prefix('#').unwrap_or(s).parse().ok())
                .ok_or(GuiParse::BadParam)
        };
        let uint = |s: Option<&str>| -> Result<usize, GuiParse> {
            s.and_then(|s| s.parse().ok()).ok_or(GuiParse::BadParam)
        };
        match verb {
            "msz" => Ok(GuiRequest::MapSize),
            "mct" => Ok(GuiRequest::MapContent),
            "tna" => Ok(GuiRequest::TeamNames),
            "sgt" => Ok(GuiRequest::TimeGet),
            "bct" => Ok(GuiRequest::Tile(uint(parts.next())?, uint(parts.next())?)),
            "ppo" => Ok(GuiRequest::PlayerPos(id(parts.next())?)),
            "plv" => Ok(GuiRequest::PlayerLevel(id(parts.next())?)),
            "pin" => Ok(GuiRequest::PlayerInv(id(parts.next())?)),
            "sst" => match parts.next().and_then(|s| s.parse::<u32>().ok()) {
                Some(t) if t > 0 => Ok(GuiRequest::TimeSet(t)),
                _ => Err(GuiParse::BadParam),
            },
            _ => Err(GuiParse::Unknown),
        }
    }

    pub fn resolve(&self, world: &World, f: &mut u32) -> Vec<String> {
        let one = |ev: GuiEvent| vec![ev.encode(world).unwrap_or_else(|| "sbp".to_string())];
        match self {
            GuiRequest::MapSize => one(GuiEvent::MapSize),
            GuiRequest::Tile(x, y) => one(GuiEvent::Tile(*x, *y)),
            GuiRequest::PlayerPos(n) => one(GuiEvent::Moved(*n)),
            GuiRequest::PlayerLevel(n) => one(GuiEvent::LevelChanged(*n)),
            GuiRequest::PlayerInv(n) => one(GuiEvent::Inventory(*n)),
            GuiRequest::MapContent => all_tiles(world),
            GuiRequest::TeamNames => all_team_names(world),
            GuiRequest::TimeGet => vec![format!("sgt {f}")],
            GuiRequest::TimeSet(t) => {
                *f = *t;
                vec![format!("sst {f}")]
            }
        }
    }
}

fn all_tiles(world: &World) -> Vec<String> {
    let mut lines = Vec::with_capacity(world.map.width * world.map.height);
    for y in 0..world.map.height {
        for x in 0..world.map.width {
            if let Some(line) = GuiEvent::Tile(x, y).encode(world) {
                lines.push(line);
            }
        }
    }
    lines
}

fn all_team_names(world: &World) -> Vec<String> {
    (0..world.teams.len())
        .filter_map(|i| GuiEvent::TeamName(i).encode(world))
        .collect()
}

pub fn init_feed(world: &World, f: u32) -> Vec<String> {
    let mut feed = Vec::new();
    if let Some(line) = GuiEvent::MapSize.encode(world) {
        feed.push(line);
    }
    feed.push(format!("sgt {f}"));
    feed.extend(all_tiles(world));
    feed.extend(all_team_names(world));
    feed.extend(
        world
            .players
            .keys()
            .filter_map(|&id| GuiEvent::NewPlayer(id).encode(world)),
    );
    feed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::game::world::names;

    #[test]
    fn map_size_encodes() {
        let w = World::new(5, 7, &names(&["t1"]), 0);
        assert_eq!(GuiEvent::MapSize.encode(&w), Some("msz 5 7".to_string()));
    }

    #[test]
    fn tile_bounds_checked() {
        let w = World::new(4, 3, &names(&["t1"]), 0);
        assert!(GuiEvent::Tile(0, 0).encode(&w).unwrap().starts_with("bct 0 0 "));
        assert_eq!(GuiEvent::Tile(4, 0).encode(&w), None);
        assert_eq!(GuiEvent::Tile(0, 3).encode(&w), None);
    }

    #[test]
    fn unknown_player_encodes_none() {
        let w = World::new(3, 3, &names(&["t1"]), 0);
        assert_eq!(GuiEvent::Moved(999).encode(&w), None);
        assert_eq!(GuiEvent::NewPlayer(999).encode(&w), None);
        assert_eq!(GuiEvent::Inventory(999).encode(&w), None);
    }

    #[test]
    fn player_events_match_spawn_state() {
        let mut w = World::new(3, 3, &names(&["t1"]), 1);
        let (id, _) = w.add_player("t1").expect("a free slot");
        let pnw = GuiEvent::NewPlayer(id).encode(&w).unwrap();
        let ppo = GuiEvent::Moved(id).encode(&w).unwrap();
        let pnw_tokens: Vec<&str> = pnw.split_whitespace().collect();
        let ppo_tokens: Vec<&str> = ppo.split_whitespace().collect();
        assert_eq!(pnw_tokens.len(), 7); // pnw #id X Y O L N
        assert_eq!(ppo_tokens.len(), 5); // ppo #id X Y O
        assert_eq!(&pnw_tokens[1..5], &ppo_tokens[1..5]); // id/X/Y/O agree
        assert_eq!(pnw_tokens[5], "1"); // level
        assert_eq!(pnw_tokens[6], "t1"); // team
        assert_eq!(
            GuiEvent::LevelChanged(id).encode(&w),
            Some(format!("plv #{id} 1"))
        );
        assert!(
            GuiEvent::Inventory(id)
                .encode(&w)
                .unwrap()
                .ends_with("10 0 0 0 0 0 0")
        );
    }

    #[test]
    fn constant_events_format() {
        let w = World::new(3, 3, &names(&["t1"]), 0);
        assert_eq!(GuiEvent::Forked(2).encode(&w).unwrap(), "pfk #2");
        assert_eq!(
            GuiEvent::EggLaid {
                egg: 9,
                player: 2,
                x: 1,
                y: 4
            }
            .encode(&w)
            .unwrap(),
            "enw #9 #2 1 4"
        );
        assert_eq!(GuiEvent::PlayerDied(3).encode(&w).unwrap(), "pdi #3");
        assert_eq!(
            GuiEvent::Collected { player: 1, res: 0 }.encode(&w).unwrap(),
            "pgt #1 0"
        );
        assert_eq!(
            GuiEvent::Broadcast {
                player: 1,
                msg: "hi there".to_string()
            }
            .encode(&w)
            .unwrap(),
            "pbc #1 hi there"
        );
        assert_eq!(
            GuiEvent::IncantationStart {
                x: 2,
                y: 3,
                level: 1,
                participants: vec![1, 4]
            }
            .encode(&w)
            .unwrap(),
            "pic 2 3 1 #1 #4"
        );
        assert_eq!(
            GuiEvent::GameWon("t1".to_string()).encode(&w).unwrap(),
            "seg t1"
        );
    }

    #[test]
    fn requests_parse_to_variants() {
        assert!(matches!(GuiRequest::parse("msz"), Ok(GuiRequest::MapSize)));
        assert!(matches!(
            GuiRequest::parse("bct 1 2"),
            Ok(GuiRequest::Tile(1, 2))
        ));
        assert!(matches!(
            GuiRequest::parse("ppo #7"),
            Ok(GuiRequest::PlayerPos(7))
        ));
        assert!(matches!(
            GuiRequest::parse("sst 50"),
            Ok(GuiRequest::TimeSet(50))
        ));
        assert!(matches!(GuiRequest::parse("sst 0"), Err(GuiParse::BadParam)));
        assert!(matches!(GuiRequest::parse("bct 1"), Err(GuiParse::BadParam)));
        assert!(matches!(
            GuiRequest::parse("ppo nope"),
            Err(GuiParse::BadParam)
        ));
        assert!(matches!(GuiRequest::parse("wat"), Err(GuiParse::Unknown)));
    }

    #[test]
    fn requests_resolve_replies() {
        let w = World::new(5, 5, &names(&["t1"]), 0);
        let mut f = 100;
        assert_eq!(GuiRequest::MapContent.resolve(&w, &mut f).len(), 25);
        assert_eq!(GuiRequest::TeamNames.resolve(&w, &mut f), vec!["tna t1"]);
        assert_eq!(GuiRequest::TimeGet.resolve(&w, &mut f), vec!["sgt 100"]);
        assert_eq!(GuiRequest::TimeSet(50).resolve(&w, &mut f), vec!["sst 50"]);
        assert_eq!(f, 50);
        assert_eq!(GuiRequest::Tile(99, 0).resolve(&w, &mut f), vec!["sbp"]);
    }

    #[test]
    fn init_feed_has_size_time_tiles_teams() {
        let w = World::new(4, 4, &names(&["t1", "t2"]), 0);
        let feed = init_feed(&w, 100);
        assert_eq!(feed[0], "msz 4 4");
        assert_eq!(feed[1], "sgt 100");
        assert_eq!(feed.iter().filter(|l| l.starts_with("bct ")).count(), 16);
        assert_eq!(feed.iter().filter(|l| l.starts_with("tna ")).count(), 2);
    }
}
