//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// main
//
mod server;

use server::Server;


fn main() {
    let server = Server::new();

    server.start();
}
