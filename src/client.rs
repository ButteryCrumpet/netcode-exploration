use std::io;
use std::net::{SocketAddr, UdpSocket};

use crate::connection::Connection;

pub struct Client {
    socket: UdpSocket,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    buffer: [u8; 1504],
    connection: Option<Connection>
}

impl Client {
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        let socket = UdpSocket::bind(local_addr).expect("Could not bind to socket");
        socket.set_nonblocking(true).unwrap();
        Client {
            socket,
            local_addr,
            remote_addr,
            buffer: [0; 1504],
            connection: None
        }
    }

    pub fn connect(&mut self) -> bool {
        self.connection = Some(Connection::new(self.local_addr, self.remote_addr));
        true
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
       match &mut self.connection {
           Some(conn) => conn.send(data, &mut self.socket),
           None => panic!("connect first")
       }
    }

    pub fn recv(&mut self) -> Result<Vec<u8>, io::Error> {
        let amt = self.socket.recv(&mut self.buffer)?;
        let data = self.buffer[..amt].to_vec();
        match &mut self.connection {
           Some(conn) => conn.receive_packet(&data),
           None => panic!("connect first")
       };
        Ok(data)
    }
}
