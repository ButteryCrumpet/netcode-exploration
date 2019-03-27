use std::io;
use std::iter;
use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::prelude::*;
use tokio::net::UdpSocket;

use crate::connection::Connection;

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
        self.socket.poll_recv_from(&mut self.buffer)
            .map(|poll| {
                let (amt, addr) = match poll {
                    Async::Ready((amt, addr)) => (amt, addr),
                    Async::NotReady => return Async::NotReady
                };

                Async::Ready((self.buffer[..amt].to_vec(), addr))
            })
    }
}

impl Future for Server {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        loop {
            let (data, addr) = match self.read() {
                Ok(Async::Ready(t)) => t,
                Ok(Async::NotReady) => return Ok(Async::NotReady),
                Err(e) =>  {
                    println!("Ahh shit.. {}", e);
                    return Ok(Async::NotReady)
                },
            };

            let num_conns = self.connections.len();
            
            if self.connections.contains_key(&addr) {
                println!("Sending {} to {} people", String::from_utf8_lossy(&data), num_conns);
                
                for (_, conn) in self.connections.iter() {
                    //conn.send(&data, &mut self.socket).unwrap();
                    conn.receive_packet(&data)
                }

            } else if num_conns < self.max_connections {
                self.connections.insert(addr, Connection::new(self.local_addr, addr));
                println!("Adding connection from address {} ({})", addr, num_conns);
            }
        }
    }
       
}