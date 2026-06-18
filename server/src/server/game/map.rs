//
// EPITECH PROJECT, 2026
// ZappyMirror
// File description:
// map (toroidal tile grid + per-tile resource counts)
//

/// Number of resource types (food + the six elevation stones).
pub const RESOURCE_COUNT: usize = 7;

/// The seven floor resources. Discriminants are the protocol indices used by the
/// GUI `bct`/`pin` messages and the `Look`/`Inventory` ordering (ARCHITECTURE.md
/// §6): `0=food 1=linemate 2=deraumere 3=sibur 4=mendiane 5=phiras 6=thystame`.
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
    /// All seven resources in protocol-index order.
    pub const ALL: [Resource; RESOURCE_COUNT] = [
        Resource::Food,
        Resource::Linemate,
        Resource::Deraumere,
        Resource::Sibur,
        Resource::Mendiane,
        Resource::Phiras,
        Resource::Thystame,
    ];

    /// Parse a resource from its protocol name (`Take`/`Set` argument).
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

    /// Protocol name, as emitted by `Look`/`Inventory`.
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

    /// Spawn density (ARCHITECTURE.md §7.1): target map quantity is
    /// `floor(width * height * density)`.
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

/// One floor tile: the count of each resource sitting on it. Players and eggs are
/// tracked by the `World` (located by their own coordinates), not stored here.
#[derive(Clone, Debug, Default)]
pub struct Tile {
    pub resources: [u32; RESOURCE_COUNT],
}

impl Tile {
    pub fn count(&self, res: Resource) -> u32 {
        self.resources[res as usize]
    }

    /// Drop `n` of `res` onto the tile.
    pub fn add(&mut self, res: Resource, n: u32) {
        self.resources[res as usize] += n;
    }

    /// Remove one `res` if present; `true` if one was taken (`Take` semantics).
    pub fn take_one(&mut self, res: Resource) -> bool {
        let slot = &mut self.resources[res as usize];
        if *slot == 0 {
            return false;
        }
        *slot -= 1;
        true
    }
}

/// The world floor: a toroidal `width × height` grid of `Tile`s, row-major.
pub struct Map {
    pub width: usize,
    pub height: usize,
    tiles: Vec<Tile>,
}

impl Map {
    /// An empty map (no resources yet — spawning is the respawn step's job).
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

    pub fn tile(&self, x: usize, y: usize) -> &Tile {
        let i = self.index(x, y);
        &self.tiles[i]
    }

    pub fn tile_mut(&mut self, x: usize, y: usize) -> &mut Tile {
        let i = self.index(x, y);
        &mut self.tiles[i]
    }
}
