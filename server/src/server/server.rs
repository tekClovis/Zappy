//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Server
//

use super::config::config::Config;
use super::map::map::Map;

pub struct Server {
    pub _config: Config,
    pub _map: Map,
}

impl Server {
    pub fn new() -> Self {
        let mut serv: Server = Server {
            _config: Config::new(4242, 10, vec!["GRAPHICAL".to_string()], 100, (10, 10)),
            _map: Map::new(10, 10),
        };
        serv._config.parse_config();
        serv._map.define_size(serv._config._world_size);
        serv
    }

    pub fn start(&self) {
        println!("Server is starting on port {}", self._config._port);
        // Additional logic to start the server can be added here
    }
}