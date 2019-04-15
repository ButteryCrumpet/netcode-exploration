use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::io;
use std::iter;
use std::net::{SocketAddr, UdpSocket};

use tokio::prelude::*;

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

    pub fn read(&mut self) -> Poll<(Vec<u8>, SocketAddr), io::Error> {
        self.socket.recv_from(&mut self.buffer).map(|poll| {
            let (amt, addr) = poll;
            Async::Ready((self.buffer[..amt].to_vec(), addr))
        })
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let (amt, addr) = match self.socket.recv_from(&mut self.buffer) {
                Ok(t) => t,
                Err(e) => {
                    println!("Ahh shit.. {}", e);
                    return Ok(Async::NotReady);
                }
            };

            match self.connections.entry(addr) {
                Occupied(_) => {
                    for (_addr, conn) in self.connections.iter_mut() {
                        conn.receive_packet(&self.buffer[..amt]);
                        let data = conn.recv_messages();
                        //print!("\r");
                        for msg in data.into_iter() {
                            //print!("{}, ", std::str::from_utf8(&msg).unwrap());
                            conn.queue_message(msg);
                        }
                        conn.send(&mut self.socket).unwrap();
                    }
                }
                Vacant(_) => {
                    if self.connections.len() < self.max_connections {
                        let mut new_con = Connection::new(self.local_addr, addr);
                        println!("New connection from {}", addr);
                        new_con.queue_message(self.buffer.clone());
                        new_con.send(&mut self.socket).unwrap();
                        self.connections.insert(addr, new_con);
                    }
                }
            };
        }
    }
}
