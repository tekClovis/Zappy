//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// main
//
mod server;

use server::{Config, Server};
use std::process::ExitCode;

fn main() -> ExitCode {
    let config = match Config::parse(std::env::args().skip(1)) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("zappy_server: {e}");
            return ExitCode::from(84);
        }
    };

    let mut server = Server::new(config);

    if let Err(e) = server.start() {
        eprintln!("Failed to start the server: {e}");
        return ExitCode::from(84);
    }
    if let Err(e) = server.run() {
        eprintln!("Server failed during execution: {e}");
        return ExitCode::from(84);
    }
    ExitCode::SUCCESS
}
