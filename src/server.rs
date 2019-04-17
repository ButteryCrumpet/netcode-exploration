use rand::prelude::*;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::io;
use std::iter;
use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::connection::Connection;

pub struct Server {
    socket: UdpSocket,
    buffer: Vec<u8>,
    connections: HashMap<SocketAddr, Connection>,
    local_addr: SocketAddr,
    max_connections: usize,
}

impl Server {
    pub fn new(
        addr: SocketAddr,
        max_packet_size: usize,
        max_connections: usize,
    ) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(&addr)?;
        let buffer: Vec<u8> = iter::repeat(0).take(max_packet_size).collect();
        let connections = HashMap::new();
        let local_addr = socket.local_addr()?;

        Ok(Server {
            socket,
            buffer,
            connections,
            local_addr,
            max_connections,
        })
    }

    pub fn run(&mut self) {
        let run_time = 19;
        let pps = 60;
        let packet_drop = 0.1;

        self.socket.set_nonblocking(true).unwrap();
        let mut rng = thread_rng();
        let mut last_sent = Instant::now();
        let mut count = 0;
        let start = Instant::now();
        loop {
            if Instant::now() - start > Duration::from_secs(run_time) {
                println!("end");
                break;
            }

            if Instant::now() - last_sent > Duration::from_millis(1000 / pps) {
                for (_, conn) in self.connections.iter_mut() {
                    conn.queue_message(&format!("ping:{}", count).into_bytes());
                    conn.send(&mut self.socket).unwrap();
                }
                count += 1;
                last_sent = Instant::now();
            }

            if let Ok((amt, addr)) = self.socket.recv_from(&mut self.buffer) {
                if rng.gen::<f32>() < packet_drop {
                    continue;
                }
                match self.connections.entry(addr) {
                    Occupied(_) => {
                        for (_addr, conn) in self.connections.iter_mut() {
                            conn.receive_packet(&self.buffer[..amt]);
                            let data = conn.recv_messages();
                            for msg in data.into_iter() {
                                println!("{}", std::str::from_utf8(&msg).unwrap());
                            }
                        }
                    }
                    Vacant(_) => {
                        if self.connections.len() < self.max_connections {
                            let mut new_con = Connection::new(self.local_addr, addr);
                            println!("New connection from {}", addr);
                            new_con.receive_packet(&self.buffer[..amt]);
                            new_con.queue_message(b"accepted\n");
                            new_con.send(&mut self.socket).unwrap();
                            self.connections.insert(addr, new_con);
                        }
                    }
                };
            };
        }
    }
}
