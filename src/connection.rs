use std::io;
use std::iter;
use std::time::Instant;
use std::net::SocketAddr;

use tokio::prelude::*;
use tokio::net::UdpSocket;

use crate::packet::Packet;

const BUFFER_SIZE: usize = 256;



enum ConnectionState {
    Connecting,
    Connected,
    Disconnected
}

pub struct Connection {
    state: ConnectionState,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    last_received_at: Instant,
    last_sent_at: Instant,
    sequence: u16,
    last_received_sequence: u16,
    ack_buffer: [Option<u16>; BUFFER_SIZE],
}

impl Connection {

    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Connection {
        Connection {
            state: ConnectionState::Connecting,
            local_addr,
            remote_addr,
            last_received_at: Instant::now(),
            last_sent_at: Instant::now(),
            sequence: 0,
            last_received_sequence: 0,
            ack_buffer: [None; BUFFER_SIZE],
        }
    }

    pub fn send(&mut self, data: &[u8], socket: &mut UdpSocket) -> Poll<usize, io::Error> {
        self.sequence += 1;
        let 
        let packet = Packet::new(self.sequence, self.last_received_sequence, );
        socket.poll_send_to(data, &self.remote_addr) 
    }

    pub fn receive_packet(&mut self, vec: &Vec<u8>) {
        let packet = Packet::from_vec(vec).unwrap();

        if packet.sequence > self.last_received_sequence {
            self.last_received_sequence = packet.sequence
        }

        for i in 0..31 {
            let seq = packet.sequence - i - 1;
            let index = seq as usize % BUFFER_SIZE;
            if self.ack_buffer[index].is_none() {
                self.ack_buffer[index] = Some(seq);
            }
        }   
    }
}