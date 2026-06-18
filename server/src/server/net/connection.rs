//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Connection (one socket: buffers, line framing, state)
//

use std::collections::VecDeque;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;

/// Where a socket sits in the handshake described in ARCHITECTURE.md §3.
#[allow(dead_code)] // `player` is read once the World layer binds AIs to drones
pub enum ConnState {
    Pending,
    Ai { player: u32 },
    Gui,
}

pub struct Connection {
    pub stream: TcpStream, // set to non-blocking on construction
    pub state: ConnState,
    in_buf: Vec<u8>,       // raw bytes not yet split into lines
    out_buf: VecDeque<u8>, // bytes waiting to be written
    pub closed: bool,
}

impl Connection {
    pub fn new(stream: TcpStream) -> std::io::Result<Self> {
        stream.set_nonblocking(true)?;
        Ok(Self {
            stream,
            state: ConnState::Pending,
            in_buf: Vec::new(),
            out_buf: VecDeque::new(),
            closed: false,
        })
    }

    pub fn send_line(&mut self, line: &str) {
        self.out_buf.extend(line.as_bytes());
        if !line.ends_with('\n') {
            self.out_buf.push_back(b'\n');
        }
    }

    pub fn wants_write(&self) -> bool {
        !self.out_buf.is_empty()
    }

    pub fn on_readable(&mut self) -> Vec<String> {
        let mut tmp = [0u8; 4096];
        loop {
            match self.stream.read(&mut tmp) {
                Ok(0) => {
                    self.closed = true;
                    break;
                } // peer closed
                Ok(n) => self.in_buf.extend_from_slice(&tmp[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.closed = true;
                    break;
                }
            }
        }
        self.take_lines()
    }

    fn take_lines(&mut self) -> Vec<String> {
        let mut lines = Vec::new();
        while let Some(pos) = self.in_buf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.in_buf.drain(..=pos).collect();
            let s = String::from_utf8_lossy(&line[..line.len() - 1]) // drop '\n'
                .trim_end_matches('\r')
                .to_string();
            lines.push(s);
        }
        lines
    }

    pub fn flush(&mut self) {
        while !self.out_buf.is_empty() {
            let bytes = self.out_buf.make_contiguous();
            match self.stream.write(bytes) {
                Ok(0) => {
                    self.closed = true;
                    break;
                }
                Ok(n) => {
                    self.out_buf.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(_) => {
                    self.closed = true;
                    break;
                }
            }
        }
    }
}
