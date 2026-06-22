//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// map
//

pub const RESOURCE_COUNT: usize = 7;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resource {
    Food = 0,
    Linemate = 1,
    Deraumere = 2,
    Sibur = 3,
    Mendiane = 4,
    Phiras = 5,
    Thystame = 6,
}

impl Resource {
    pub const ALL: [Resource; RESOURCE_COUNT] = [
        Resource::Food,
        Resource::Linemate,
        Resource::Deraumere,
        Resource::Sibur,
        Resource::Mendiane,
        Resource::Phiras,
        Resource::Thystame,
    ];

    pub fn from_name(name: &str) -> Option<Resource> {
        match name {
            "food" => Some(Resource::Food),
            "linemate" => Some(Resource::Linemate),
            "deraumere" => Some(Resource::Deraumere),
            "sibur" => Some(Resource::Sibur),
            "mendiane" => Some(Resource::Mendiane),
            "phiras" => Some(Resource::Phiras),
            "thystame" => Some(Resource::Thystame),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Resource::Food => "food",
            Resource::Linemate => "linemate",
            Resource::Deraumere => "deraumere",
            Resource::Sibur => "sibur",
            Resource::Mendiane => "mendiane",
            Resource::Phiras => "phiras",
            Resource::Thystame => "thystame",
        }
    }

    pub fn density(self) -> f64 {
        match self {
            Resource::Food => 0.5,
            Resource::Linemate => 0.3,
            Resource::Deraumere => 0.15,
            Resource::Sibur => 0.1,
            Resource::Mendiane => 0.1,
            Resource::Phiras => 0.08,
            Resource::Thystame => 0.05,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub resources: [u32; RESOURCE_COUNT],
}

impl Tile {
    pub fn count(&self, res: Resource) -> u32 {
        self.resources[res as usize]
    }

    pub fn add(&mut self, res: Resource, n: u32) {
        self.resources[res as usize] += n;
    }

    pub fn take(&mut self, res: Resource, n: u32) {
        self.resources[res as usize] -= n;
    }

    pub fn take_one(&mut self, res: Resource) -> bool {
        let slot = &mut self.resources[res as usize];
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }
}

/// Signed shortest distance from `a` to `b` on a ring of size `n`, in `(-n/2, n/2]`.
fn ring_delta(a: usize, b: usize, n: usize) -> isize {
    let n = n as isize;
    let m = (b as isize - a as isize).rem_euclid(n);
    if m * 2 > n { m - n } else { m }
}

pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Map {
            width,
            height,
            tiles: vec![Tile::default(); width * height],
        }
    }

    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    pub fn wrap(&self, x: isize, y: isize) -> (usize, usize) {
        let w = self.width as isize;
        let h = self.height as isize;
        (x.rem_euclid(w) as usize, y.rem_euclid(h) as usize)
    }

    /// Shortest signed offset from `from` to `to` on the torus, each component
    /// reduced to `(-n/2, n/2]`. `(0, 0)` means the same tile.
    pub fn shortest_offset(&self, from: (usize, usize), to: (usize, usize)) -> (isize, isize) {
        (
            ring_delta(from.0, to.0, self.width),
            ring_delta(from.1, to.1, self.height),
        )
    }

    pub fn tile(&self, x: usize, y: usize) -> &Tile {
        let i = self.index(x, y);
        &self.tiles[i]
    }

    pub fn tile_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        let i = self.index(x, y);
        &mut self.tiles[i]
    }

    pub fn total(&self, res: Resource) -> u32 {
        self.tiles.iter().map(|t| t.count(res)).sum()
    }
}
