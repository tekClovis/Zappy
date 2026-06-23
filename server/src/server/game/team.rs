//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// team (slots/eggs + connected players)
//

#[derive(Debug, PartialEq, Eq)]
pub enum JoinError {
    UnknownTeam,
    TeamFull,
}

/// An unhatched slot on the floor at `(x, y)`. Its owning team is implied by the
/// `Team` that holds it. Laid at startup (one per client slot) or by `Fork`;
/// consumed on connect.
#[derive(Debug)]
pub struct Egg {
    pub id: u32,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct Team {
    pub name: String,
    /// Unhatched eggs; one is consumed per `connect`, one added per `Fork`.
    pub eggs: Vec<Egg>,
    pub players: Vec<u32>,
}

impl Team {
    pub fn new(name: String) -> Self {
        Team {
            name,
            eggs: Vec::new(),
            players: Vec::new(),
        }
    }

    pub fn lay_egg(&mut self, id: u32, x: usize, y: usize) {
        self.eggs.push(Egg { id, x, y });
    }

    pub fn hatch(&mut self) -> Option<Egg> {
        if self.eggs.is_empty() {
            None
        } else {
            Some(self.eggs.remove(0))
        }
    }

    pub fn remove_egg(&mut self, id: u32) -> Option<Egg> {
        let i = self.eggs.iter().position(|e| e.id == id)?;
        Some(self.eggs.remove(i))
    }

    pub fn eggs_at(&self, x: usize, y: usize) -> impl Iterator<Item = u32> + '_ {
        self.eggs
            .iter()
            .filter(move |e| e.x == x && e.y == y)
            .map(|e| e.id)
    }

    pub fn remaining_eggs(&self) -> usize {
        self.eggs.len()
    }
}
