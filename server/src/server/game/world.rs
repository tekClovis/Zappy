//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// world
//

pub struct World {
    pub w_h: (usize, usize)
}

impl World {
    pub fn new (width: usize, height: usize) -> Self{
        World {w_h: (width, height)}
    }
}