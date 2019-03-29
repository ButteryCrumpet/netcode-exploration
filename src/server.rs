use std::io;
use std::time;
use std::iter;
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::thread;

use tokio::prelude::*;

use crate::connection::Connection;
use crate::packet::Packet;

pub struct Server {
    socket: UdpSocket,
    buffer: Vec<u8>,
    connections: HashMap<SocketAddr, Connection>,
    local_addr: SocketAddr,
    max_connections: usize
}

impl Server {

    pub fn new(addr: SocketAddr, max_packet_size: usize, max_connections: usize) -> Result<Self, io::Error> {
        let socket = UdpSocket::bind(&addr)?;
        let buffer: Vec<u8> = iter::repeat(0).take(max_packet_size).collect();
        let connections = HashMap::new();
        let local_addr = socket.local_addr()?;

        Ok(Server {
            socket,
            buffer,
            connections,
            local_addr,
            max_connections
        })
    }

    pub fn read(&mut self) -> Poll<(Vec<u8>, SocketAddr), io::Error> {
        self.socket.recv_from(&mut self.buffer)
            .map(|poll| {
                let (amt, addr) = poll;
                Async::Ready((self.buffer[..amt].to_vec(), addr))
            })
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {

        let addr = "127.0.0.1:12346".parse().unwrap();
        if self.local_addr != addr {
            let mut new_con = Connection::new(self.local_addr, addr);
            new_con.send(&self.buffer, &mut self.socket).unwrap();
            self.connections.insert(addr, new_con);
        }

        let start = time::Instant::now();

        loop {
            
            thread::sleep(time::Duration::from_millis(33));

            let (amt, addr) = match self.socket.recv_from(&mut self.buffer) {
                Ok(t) => t,
                Err(e) =>  {
                    println!("Ahh shit.. {}", e);
                    return Ok(Async::NotReady)
                },
            };

            let num_conns = self.connections.len();
            
            if self.connections.contains_key(&addr) {

                for (ad, conn) in self.connections.iter_mut() {
                    let data = conn.receive_packet(&self.buffer);
                    conn.send(&data, &mut self.socket).unwrap();
                }

            } else if num_conns < self.max_connections {
                let mut new_con = Connection::new(self.local_addr, addr);
                println!("new connection");
                new_con.send(&self.buffer, &mut self.socket).unwrap();
                self.connections.insert(addr, new_con);
            }

            if time::Instant::now() - start > time::Duration::from_secs(20) {
                println!("{} done", self.local_addr);
                return Ok(Async::Ready(()));
            }
        }
    }
       
}