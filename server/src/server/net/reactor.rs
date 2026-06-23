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

use crate::server::game::command::{Command, EnqueueError};
use crate::server::game::gui::{self, GuiParse, GuiRequest};
use crate::server::game::player::{STARVE_INTERVAL_UNITS, StarveResult};
use crate::server::game::team::JoinError;
use crate::server::game::world::{Target, World};
use crate::server::net::connection::{ConnState, Connection};
use crate::server::net::scheduler::{Event, Scheduler};

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
            self.deliver_outbox();
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
            Route::Gui => self.handle_gui_request(fd, line),
        }
    }

    fn handle_gui_request(&mut self, fd: i32, line: String) {
        let replies = match GuiRequest::parse(&line) {
            Ok(req) => req.resolve(&self.world, &mut self.f),
            Err(GuiParse::BadParam) => vec!["sbp".to_string()],
            Err(GuiParse::Unknown) => vec!["suc".to_string()],
        };
        if let Some(conn) = self.conns.get_mut(&fd) {
            for reply in replies {
                conn.send_line(&reply);
            }
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
        let Some((command_id, cost)) = self.world.start_next(player) else {
            return;
        };
        if !self.world.current_is_incantation(player) {
            self.sched
                .schedule_units(cost, self.f, Event::ActionDone { player, command_id });
            return;
        }
        // Incantation is two-phase: validate now, then fire `IncantationDone`.
        match Command::incantation_start(&mut self.world, player) {
            Some((level, tile, participants)) => {
                self.sched.schedule_units(
                    cost,
                    self.f,
                    Event::IncantationDone {
                        tile,
                        level,
                        participants,
                    },
                );
            }
            None => {
                self.world.abort_current(player);
                self.world
                    .outbox
                    .push((Target::Player(player), "ko".to_string()));
                self.maybe_start(player);
            }
        }
    }

    fn conn_for_player(&mut self, player: u32) -> Option<&mut Connection> {
        self.conns
            .values_mut()
            .find(|c| matches!(c.state, ConnState::Ai { player: p } if p == player))
    }

    fn handle_handshake(&mut self, fd: i32, line: String) {
        let (width, height) = (self.world.map.width, self.world.map.height);

        if line == "GRAPHIC" {
            let feed = gui::init_feed(&self.world, self.f);
            if let Some(conn) = self.conns.get_mut(&fd) {
                conn.state = ConnState::Gui;
                for feed_line in feed {
                    conn.send_line(&feed_line);
                }
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
                self.sched
                    .schedule_units(STARVE_INTERVAL_UNITS, self.f, Event::Starve { player });
            }
            Err(err) => {
                if let Some(conn) = self.conns.get_mut(&fd) {
                    let reply = match err {
                        JoinError::UnknownTeam => "ko",
                        JoinError::TeamFull => "0",
                    };
                    conn.send_final(reply);
                }
            }
        }
    }

    fn deliver_outbox(&mut self) {
        for (target, line) in self.world.take_outbox() {
            match target {
                Target::Player(p) => {
                    if let Some(conn) = self.conn_for_player(p) {
                        conn.send_line(&line);
                    }
                }
                Target::AllGui => {
                    for conn in self.conns.values_mut() {
                        if matches!(conn.state, ConnState::Gui) {
                            conn.send_line(&line);
                        }
                    }
                }
            }
        }
    }

    fn handle_due_events(&mut self) {
        let now = Instant::now();
        while let Some(event) = self.sched.pop_due(now) {
            match event {
                Event::RespawnResources => {
                    self.world.respawn_resources();
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
                Event::IncantationDone {
                    tile,
                    level,
                    participants,
                } => {
                    Command::incantation_finish(&mut self.world, tile, level, &participants);
                    for p in participants {
                        self.maybe_start(p);
                    }
                }
                Event::Starve { player } => match self.world.consume_food(player) {
                    StarveResult::Survived => {
                        self.sched.schedule_units(
                            STARVE_INTERVAL_UNITS,
                            self.f,
                            Event::Starve { player },
                        );
                    }
                    StarveResult::Died => {
                        if let Some(conn) = self.conn_for_player(player) {
                            conn.send_final("dead");
                        }
                    }
                    StarveResult::Gone => {}
                },
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
