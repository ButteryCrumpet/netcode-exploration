use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, UdpSocket};

use crate::connection::Connection;

pub struct Client {
    socket: UdpSocket,
    local_addr: SocketAddr,
    remote_addr: Option<SocketAddr>,
    buffer: [u8; 1504],
    connection: Option<Connection>,
    message_queue: VecDeque<Vec<u8>>,
}

impl Client {
    pub fn new(local_addr: SocketAddr) -> Self {
        let socket = UdpSocket::bind(local_addr).expect("Could not bind to socket");
        socket.set_nonblocking(true).unwrap();
        Client {
            socket,
            local_addr,
            remote_addr: None,
            buffer: [0; 1504],
            connection: None,
            message_queue: VecDeque::new(),
        }
    }

    pub fn connect(&mut self, remote: SocketAddr) -> bool {
        self.remote_addr = Some(remote);
        self.connection = Some(Connection::new(self.local_addr, self.remote_addr.unwrap()));
        true
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, std::io::Error> {
        match &mut self.connection {
            Some(conn) => conn.send(data, &mut self.socket),
            None => panic!("connect first"),
        }
    }

    pub fn send_next(&mut self) -> Result<Option<usize>, std::io::Error> {
        if let Some(data) = self.message_queue.pop_front() {
            return match self.send(&data) {
                Ok(u) => Ok(Some(u)),
                Err(e) => Err(e),
            };
        }
        Ok(None)
    }

    pub fn recv(&mut self) -> Result<Vec<u8>, io::Error> {
        let amt = self.socket.recv(&mut self.buffer)?;
        let data = self.buffer[..amt].to_vec();
        let recv = match &mut self.connection {
            Some(conn) => conn.receive_packet(&data),
            None => panic!("connect first"),
        };

        Ok(recv)
    }

    pub fn queue_message(&mut self, message: Vec<u8>) {
        self.message_queue.push_back(message);
    }
}
