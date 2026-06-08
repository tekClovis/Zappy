//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// client
//

use std::net::TcpStream;
use std::io::Write;
use std::io::Read;
use std::io::ErrorKind;

pub trait Client {
    fn new(stream: TcpStream) -> Self;
    fn get_stream(&mut self) -> &mut TcpStream;
    fn read(&mut self) -> std::io::Result<String> {
        let mut buffer = [0; 1024];

        match self.get_stream().read(&mut buffer) {
            Ok(0) => Err(std::io::Error::new(ErrorKind::UnexpectedEof, "Client disconnected")),
            Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(String::new()),
            Ok(n) => Ok(String::from_utf8_lossy(&buffer[..n]).to_string()),
            Err(e) => Err(e),
        }
    }
    fn write (&mut self, data: &str) -> std::io::Result<()> {
        self.get_stream().write_all(data.as_bytes())
    }
}

pub struct AIClient {
    _team_name: String,
    _stream: TcpStream,
}

impl Client for AIClient {
    fn new(stream: TcpStream) -> Self {
        let client = AIClient {
            _stream: stream,
            _team_name: String::new(),
        };
        client
    }

    fn get_stream(&mut self) -> &mut TcpStream {
        &mut self._stream
    }
}