//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Server
//

use super::config::Config;
use super::game::world::World;
use super::net::reactor::Reactor;
use std::io::Result;
use std::net::TcpListener;

pub struct Server {
    config: Config,
    listener: Option<TcpListener>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        Server {
            config,
            listener: None,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        println!("Server configuration:");
        println!("Port: {}", self.config.port);
        println!("Max Clients: {}", self.config.clients);
        println!("Teams Names: {:?}", self.config.names);
        println!("Time Unit: {}", self.config.frequency);
        println!("World Size: {}x{}", self.config.x, self.config.y);

        let listener = TcpListener::bind(format!("127.0.0.1:{}", self.config.port))?;
        listener.set_nonblocking(true)?;
        println!("Server started on port {}", listener.local_addr()?.port());
        self.listener = Some(listener);
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        let listener = match self.listener.take() {
            Some(listener) => listener,
            None => {
                eprintln!("Server is not running. Please start the server first.");
                return Ok(());
            }
        };

        let map = World::new(
            self.config.x,
            self.config.y,
            &self.config.names,
            self.config.clients,
        );
        let mut reactor = Reactor::new(listener, self.config.frequency, map)?;
        reactor.run()
    }
}
