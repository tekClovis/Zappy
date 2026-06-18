//
// EPITECH PROJECT, 2026
// Zappy
// File description:
// Reactor (poll loop + fd registry) — replaces the old busy loop
//

use std::collections::HashMap;
use std::io::{ErrorKind, Result};
use std::net::TcpListener;
use std::os::fd::AsRawFd;
use std::time::Instant;

use crate::server::game::command::EnqueueError;
use crate::server::game::team::JoinError;
use crate::server::game::world::World;
use crate::server::net::connection::{ConnState, Connection};
use crate::server::scheduler::{Event, Scheduler};

enum Route {
    Handshake,
    Ai(u32),
    Gui,
}

pub struct Reactor {
    listener: TcpListener,
    conns: HashMap<i32, Connection>,
    fds: Vec<libc::pollfd>,
    sched: Scheduler,
    world: World,
    f: u32,
}

impl Reactor {
    pub fn new(listener: TcpListener, f: u32, world: World) -> Result<Self> {
        listener.set_nonblocking(true)?;
        let mut sched = Scheduler::new();
        sched.schedule_units(20, f, Event::RespawnResources);
        Ok(Self {
            fds: vec![libc::pollfd {
                fd: listener.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            }],
            listener,
            conns: HashMap::new(),
            sched,
            world,
            f,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            self.refresh_events();

            let timeout_ms = match self.sched.next_timeout() {
                Some(d) => d.as_millis().min(i32::MAX as u128) as i32,
                None => -1,
            };

            let nfds = self.fds.len() as libc::nfds_t;
            let n = unsafe { libc::poll(self.fds.as_mut_ptr(), nfds, timeout_ms) };
            if n < 0 {
                let err = std::io::Error::last_os_error();
                if err.kind() == ErrorKind::Interrupted {
                    continue;
                }
                return Err(err);
            }

            self.handle_io();
            self.handle_due_events();
            self.reap_closed();
        }
    }

    fn refresh_events(&mut self) {
        let listener_fd = self.listener.as_raw_fd();
        for i in 0..self.fds.len() {
            let fd = self.fds[i].fd;
            let mut events = libc::POLLIN;
            if fd != listener_fd && self.conns.get(&fd).is_some_and(|c| c.wants_write()) {
                events |= libc::POLLOUT;
            }
            self.fds[i].events = events;
            self.fds[i].revents = 0;
        }
    }

    fn handle_io(&mut self) {
        let listener_fd = self.listener.as_raw_fd();
        let mut inbound: Vec<(i32, Vec<String>)> = Vec::new();

        let len = self.fds.len();
        for i in 0..len {
            let pfd = self.fds[i];
            if pfd.revents == 0 {
                continue;
            }
            if pfd.fd == listener_fd {
                self.accept_new();
                continue;
            }
            if let Some(conn) = self.conns.get_mut(&pfd.fd) {
                if pfd.revents & (libc::POLLIN | libc::POLLHUP) != 0 {
                    let lines = conn.on_readable();
                    if !lines.is_empty() {
                        inbound.push((pfd.fd, lines));
                    }
                }
                if pfd.revents & libc::POLLOUT != 0 {
                    conn.flush();
                }
            }
        }

        for (fd, lines) in inbound {
            for line in lines {
                self.process_line(fd, line);
            }
        }
    }

    fn accept_new(&mut self) {
        loop {
            match self.listener.accept() {
                Ok((stream, _addr)) => {
                    if let Ok(mut conn) = Connection::new(stream) {
                        let fd = conn.stream.as_raw_fd();
                        conn.send_line("WELCOME");
                        self.conns.insert(fd, conn);
                        self.fds.push(libc::pollfd {
                            fd,
                            events: libc::POLLIN,
                            revents: 0,
                        });
                    }
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
    }

    fn process_line(&mut self, fd: i32, line: String) {
        let route = match self.conns.get(&fd).map(|c| &c.state) {
            Some(ConnState::Pending) => Route::Handshake,
            Some(ConnState::Ai { player }) => Route::Ai(*player),
            Some(ConnState::Gui) => Route::Gui,
            None => return,
        };
        match route {
            Route::Handshake => self.handle_handshake(fd, line),
            Route::Ai(player) => self.handle_command(fd, player, line),
            Route::Gui => {} // GUI requests (mct, ...) land later
        }
    }

    fn handle_command(&mut self, fd: i32, player: u32, line: String) {
        match self.world.enqueue_command(player, &line) {
            Ok(()) => self.maybe_start(player),
            Err(EnqueueError::BadCommand) => {
                if let Some(conn) = self.conns.get_mut(&fd) {
                    conn.send_line("ko");
                }
            }
            Err(EnqueueError::QueueFull) => {}
        }
    }

    fn maybe_start(&mut self, player: u32) {
        if let Some((command_id, cost)) = self.world.start_next(player) {
            self.sched
                .schedule_units(cost, self.f, Event::ActionDone { player, command_id });
        }
    }

    fn conn_for_player(&mut self, player: u32) -> Option<&mut Connection> {
        self.conns
            .values_mut()
            .find(|c| matches!(c.state, ConnState::Ai { player: p } if p == player))
    }

    fn handle_handshake(&mut self, fd: i32, line: String) {
        let (width, height) = self.world.w_h;

        if line == "GRAPHIC" {
            if let Some(conn) = self.conns.get_mut(&fd) {
                conn.state = ConnState::Gui;
                // GUI init feed
            }
            return;
        }

        match self.world.add_player(&line) {
            Ok((player, remaining)) => {
                if let Some(conn) = self.conns.get_mut(&fd) {
                    conn.state = ConnState::Ai { player };
                    conn.send_line(&remaining.to_string());
                    conn.send_line(&format!("{} {}", width, height));
                }
            }
            Err(err) => {
                if let Some(conn) = self.conns.get_mut(&fd) {
                    let reply = match err {
                        JoinError::UnknownTeam => "ko",
                        JoinError::TeamFull => "0",
                    };
                    conn.send_line(reply);
                    conn.flush();
                    conn.closed = true;
                }
            }
        }
    }

    fn handle_due_events(&mut self) {
        let now = Instant::now();
        while let Some(event) = self.sched.pop_due(now) {
            match event {
                Event::RespawnResources => {
                    self.sched
                        .schedule_units(20, self.f, Event::RespawnResources);
                }
                Event::ActionDone { player, command_id } => {
                    if let Some(reply) = self.world.finish_command(player, command_id) {
                        if let Some(conn) = self.conn_for_player(player) {
                            conn.send_line(&reply);
                        }
                    }
                    self.maybe_start(player);
                }
                Event::IncantationDone { tile, level } => {
                    let _ = (tile, level);
                }
                Event::Starve { player } => {
                    let _ = player;
                }
            }
        }
    }

    fn reap_closed(&mut self) {
        if !self.conns.values().any(|c| c.closed) {
            return;
        }

        let gone: Vec<u32> = self
            .conns
            .values()
            .filter_map(|c| match c.state {
                ConnState::Ai { player } if c.closed => Some(player),
                _ => None,
            })
            .collect();
        for player in gone {
            self.world.remove_player(player);
        }

        let mut fds = std::mem::take(&mut self.fds);
        fds.retain(|pfd| self.conns.get(&pfd.fd).is_none_or(|c| !c.closed));
        self.fds = fds;
        self.conns.retain(|_, c| !c.closed);
    }
}
