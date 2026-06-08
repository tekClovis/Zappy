//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Config
//
use std::env;

pub struct Config {
    pub _port: u16,
    pub _max_clients: usize,
    pub _teams_names: Vec<String>,
    pub _frequency: u32,
    pub _world_size: (usize, usize),
}

impl Config {
    pub fn new(port: u16, max_clients: usize, teams_names: Vec<String>, frequency: u32, world_size: (usize, usize)) -> Self {
        Config {
            _port: port,
            _max_clients: max_clients,
            _teams_names: teams_names,
            _frequency: frequency,
            _world_size: world_size,
        }
    }

    pub fn parse_config(&self) {
        let args: Vec<String> = env::args().collect();
        for arg in args.iter() {
            println!("Argument: {}", arg);
        }
    }
}