//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// main
//
mod server;

use server::Server;

fn main() -> std::io::Result<()> {
    let mut server = Server::new();

    if let Err(e) = server.start() {
        eprintln!("Failed to start the server: {}", e);
        return Err(e);
    }
    if let Err(e) = server.run() {
        eprintln!("Server Failed during execution: {}", e);
        return Err(e);
    }
    Ok(())
}
