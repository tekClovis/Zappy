//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Map
//

pub struct Map {
    pub width: usize,
    pub height: usize,
    // Additional fields for the map can be added here
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Map { width, height }
    }
    pub fn define_size(&mut self,(width, height): (usize, usize)) {
        self.width = width;
        self.height = height;
    }   
}