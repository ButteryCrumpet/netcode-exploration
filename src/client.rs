use std::collections::VecDeque;
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc::TryRecvError;
use std::thread;

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

    pub fn connect(&mut self, remote: SocketAddr) -> io::Result<()> {
        self.remote_addr = Some(remote);
        let mut new_conn = Connection::new(self.local_addr, self.remote_addr.unwrap());
        while let Some(message) = self.message_queue.pop_front() {
            new_conn.queue_message(&message);
        }
        self.connection = Some(new_conn);
        Ok(())
    }

    pub fn send_next(&mut self) -> Result<usize, std::io::Error> {
        if let Some(conn) = &mut self.connection {
            return conn.send(&mut self.socket);
        }
        panic!("connect first");
    }

    pub fn recv(&mut self) -> Result<usize, TryRecvError> {
        if let Ok((amt, addr)) = self.socket.recv_from(&mut self.buffer) {
            if addr != self.remote_addr.unwrap() {
                return Ok(0);
            }
            let data = self.buffer[..amt].to_vec();
            match &mut self.connection {
                Some(conn) => conn.receive_packet(&data),
                None => panic!("connect first"),
            };
            Ok(amt)
        } else {
            Err(TryRecvError::Empty)
        }
    }

    pub fn queue_message(&mut self, message: Vec<u8>) {
        match &mut self.connection {
            Some(conn) => conn.queue_message(&message),
            None => self.message_queue.push_back(message),
        }
    }

    pub fn recv_messages(&mut self) -> Option<Vec<Vec<u8>>> {
        if let Some(conn) = &mut self.connection {
            return Some(conn.recv_messages());
        }
        None
    }
}
