use std::io;
use std::cmp::min;
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

#[derive(Copy, Clone)]
enum PacketState {
    Acked(u16),
    UnAcked(u16)
}


pub struct Connection {
    state: ConnectionState,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    last_received_at: Instant,
    last_sent_at: Instant,
    sequence: u16,
    last_received_sequence: u16,
    recv_ack_buffer: [Option<u16>; BUFFER_SIZE],
    sent_ack_buffer: [Option<PacketState>; BUFFER_SIZE],
    recv_packets: u32,
    acked_packets: u32,
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
            recv_ack_buffer: [None; BUFFER_SIZE],
            sent_ack_buffer: [None; BUFFER_SIZE],
            recv_packets: 0,
            acked_packets: 0,
        }
    }

    pub fn send(&mut self, data: &[u8], socket: &mut UdpSocket) -> Poll<usize, io::Error> {
        use PacketState::{Acked, UnAcked};

        let index = self.sequence as usize % BUFFER_SIZE;
        self.sent_ack_buffer[index] = Some(UnAcked(self.sequence));
        
        let mut acks: Vec<u16> = Vec::with_capacity(32);
        for i in 0..min(self.last_received_sequence, 32) {
            let seq = self.last_received_sequence - i;
            let index = seq as usize % BUFFER_SIZE;
            if self.recv_ack_buffer[index].is_some() {
                let buffered = self.recv_ack_buffer[index].unwrap();
                if seq == buffered {
                    acks.push(seq)
                }
            }
        }

        let packet = Packet::new(self.sequence, self.last_received_sequence, acks);
        let send = socket.poll_send_to(&packet.into_vec(), &self.remote_addr);

        self.sequence = self.sequence.wrapping_add(1);
        self.last_sent_at = Instant::now();

        send
    }

    pub fn receive_packet(&mut self, vec: &Vec<u8>) {
        use PacketState::{Acked, UnAcked};

        let packet = Packet::from_vec(vec).unwrap();
        self.recv_packets = self.recv_packets.wrapping_add(1);
        // Update last received packet sequence number
        if packet.sequence > self.last_received_sequence {
            self.last_received_sequence = packet.sequence
        }

        // Update received at time
        self.last_received_at = Instant::now();

        // Buffer sequence number for sending back acks
        let index = packet.sequence as usize % BUFFER_SIZE;
        self.recv_ack_buffer[index] = Some(packet.sequence);


        // Confirm received acks
        for seq in packet.acks.iter() {
            let sn = *seq as usize;
            let index = sn % BUFFER_SIZE;
            self.sent_ack_buffer[index] = match self.sent_ack_buffer[index] {
                Some(Acked(s)) => Some(Acked(s)),
                Some(UnAcked(s)) => {
                    if s != sn as u16 {
                        println!("Packet {} was lost, replace by {} at index {}", s, sn, index);
                    }
                    self.acked_packets = self.acked_packets.wrapping_add(1);
                    Some(Acked(sn as u16))
                },
                None => None,
            };
        }

        if self.sequence % 10 == 0 {
            println!("recv: {} - acked: {}", self.recv_packets, self.acked_packets);
        }

    }
}