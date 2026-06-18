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

#[derive(Debug)]
pub struct Team {
    pub name: String,
    pub slots: usize,
    pub players: Vec<u32>,
}

impl Team {
    pub fn new(name: String, slots: usize) -> Self {
        Team {
            name,
            slots,
            players: Vec::new(),
        }
    }

    pub fn has_free_slot(&self) -> bool {
        self.slots > 0
    }

    pub fn remaining(&self) -> usize {
        self.slots
    }
}
