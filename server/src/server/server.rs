//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Server
//

use super::config::Config;
use super::map::Map;
use super::client::{AIClient, Client};
use std::io::{ Result, ErrorKind};
use std::net::TcpListener;

pub struct Server {
    _config: Config,
    _map: Map,
    _listener: Option<TcpListener>,
    _clients: Vec<AIClient>,
}

impl Server {
    pub fn new() -> Self {
        let mut serv: Server = Server {
            _config: Config::new(),
            _map: Map::new(10, 10),
            _listener: None,
            _clients: Vec::new(),
        };
        serv._map.define_size((serv._config.x, serv._config.y));
        serv._config.names.push("GRAPHICAL".to_string());
        serv
    }


    pub fn start(&mut self) -> Result<()> {
        for (_config, _map) in [(&self._config, &self._map)] {
            println!("Server configuration:");
            println!("Port: {}", _config.port);
            println!("Max Clients: {}", _config.clients);
            println!("Teams Names: {:?}", _config.names);
            println!("Time Unit: {} ms", _config.frequency);
            println!("World Size: {}x{}", _config.x, _config.y);
        }

        let listener = TcpListener::bind(format!("127.0.0.1:{}", self._config.port))?;

        self._listener = Some(listener);
        println!("Server started on porta {}", self._listener.as_ref().unwrap().local_addr()?.port());
        self._listener.as_ref().unwrap().set_nonblocking(true)?;
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        if self._listener.is_none() {
            eprintln!("Server is not running. Please start the server first.");
            return Ok(());
        }
        let listener = self._listener.as_ref().unwrap();

        loop {
            match listener.accept() {
                Ok((stream, addr)) => {
                    println!("New client: {}", addr);
                    stream.set_nonblocking(true).unwrap();
                    let mut client: AIClient = AIClient::new(stream);
                    client.write(format!("Successfully connected to the server at {}:{}", listener.local_addr()?.ip(), listener.local_addr()?.port()).as_str())?;
                    self._clients.push(client);
                }
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    // No incoming connection, continue with other tasks
                }
                Err(e) => eprintln!("Accept error: {}", e),
            }

            self._clients.iter_mut().for_each(|client| {
                match client.read() {
                    Ok(string ) => println!("{}", string),
                    Err(_) => println!("A client has disconnected"),
                }
            });
        }
    }
}