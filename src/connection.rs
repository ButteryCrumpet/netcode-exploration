use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

use crate::message_queue::MessageQueue;
use crate::packet::Packet;

const BUFFER_SIZE: usize = 128;

#[derive(Copy, Clone, Debug)]
struct PacketData {
    seq: u16,
    sent_time: Instant,
}

#[derive(Copy, Clone, Debug)]
enum PacketState {
    Acknowledged(PacketData),
    UnAcknowledged(PacketData),
}

pub struct Connection {
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    last_received_at: Instant,
    last_sent_at: Instant,
    sequence: u16,
    last_received_sequence: u16,
    recv_ack_buffer: [Option<u16>; BUFFER_SIZE],
    sent_ack_buffer: [Option<PacketState>; BUFFER_SIZE],
    message_queue: MessageQueue,
    recv_packets: u32,
    acked_packets: u32,
    lost_packets: u32,
    sent_packets: u32,
    rtt: f32,
}

impl Connection {
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Connection {
        Connection {
            local_addr,
            remote_addr,
            last_received_at: Instant::now(),
            last_sent_at: Instant::now(),
            sequence: 0,
            last_received_sequence: 0,
            recv_ack_buffer: [None; BUFFER_SIZE],
            sent_ack_buffer: [None; BUFFER_SIZE],
            message_queue: MessageQueue::new(),
            recv_packets: 0,
            acked_packets: 0,
            lost_packets: 0,
            sent_packets: 0,
            rtt: 0.0,
        }
    }

    pub fn queue_message(&mut self, message: &[u8]) {
        self.message_queue.queue_message(message);
    }

    pub fn send(&mut self, socket: &mut UdpSocket) -> Result<usize, std::io::Error> {
        use PacketState::UnAcknowledged;

        // Set sent packer buffer to ack them when needed
        let index = self.sequence as usize % BUFFER_SIZE;

        // if unacked packet exists at location sequence has wrapped
        // round and packet has been lost. ttl is send_rate / BUFFER_SIZE
        // so buffer of 128 with a 60pps means a ~470ms ttl
        if let Some(UnAcknowledged(_lost_packet)) = self.sent_ack_buffer[index] {
            self.lost_packets = self.lost_packets.wrapping_add(1);
        }

        self.sent_ack_buffer[index] = Some(UnAcknowledged(PacketData {
            seq: self.sequence,
            sent_time: Instant::now(),
        }));

        // Get last 32 received packets and add them to acks if they exist
        let mut acks: Vec<u16> = Vec::with_capacity(32);
        for i in 0..32 {
            let seq = self.last_received_sequence.wrapping_sub(i);
            let index = seq as usize % BUFFER_SIZE;

            if let Some(buffered) = self.recv_ack_buffer[index] {
                if seq == buffered {
                    acks.push(seq);
                }
            }
        }

        let data = self.message_queue.send_next(self.sequence, 1200);
        let packet = Packet::new(self.sequence, self.last_received_sequence, acks, data);

        let sent = socket.send_to(&packet.into_vec(), &self.remote_addr)?;

        self.sequence = self.sequence.wrapping_add(1);
        self.sent_packets = self.sent_packets.wrapping_add(1);
        self.last_sent_at = Instant::now();

        Ok(sent)
    }

    pub fn receive_packet(&mut self, data: &[u8]) {
        use PacketState::{Acknowledged, UnAcknowledged};

        let packet = Packet::from_slice(data).unwrap();
        self.recv_packets = self.recv_packets.wrapping_add(1);

        // Update last received packet sequence number if it is within
        // window of half u16::MAX
        if is_recent(packet.sequence, self.last_received_sequence) {
            self.last_received_sequence = packet.sequence
        }

        // Update received at time
        self.last_received_at = Instant::now();

        // Buffer sequence number for sending back acks
        let index = packet.sequence as usize % BUFFER_SIZE;
        self.recv_ack_buffer[index] = Some(packet.sequence);

        // Receive messages into message queue
        self.message_queue.recv_messages(&packet.data);

        // Confirm received acks
        for seq in packet.acks.iter() {
            let sn = *seq as usize;
            let index = sn % BUFFER_SIZE;

            // If we we have sent a packet and it is currently unacked
            // we need to set it to acked.
            if let Some(UnAcknowledged(pdata)) = self.sent_ack_buffer[index] {
                self.sent_ack_buffer[index] = Some(Acknowledged(pdata));
                self.acked_packets = self.acked_packets.wrapping_add(1);

                // Ack the message queue
                self.message_queue.acknowledge(pdata.seq);

                self.rtt = smoothed_average(self.rtt, self.last_received_at - pdata.sent_time);
            };
        }
    }

    pub fn recv_messages(&mut self) -> Vec<Vec<u8>> {
        self.message_queue.recv_next_all()
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        println!(
            "sent {}\nrecv {}\nacked {}\nlost {}\nrecent_recv {}\npacket rtt {}ms\n",
            self.sent_packets,
            self.recv_packets,
            self.acked_packets,
            self.lost_packets,
            self.last_received_sequence,
            self.rtt,
        );
    }
}

// Static Helpers

fn is_recent(new: u16, old: u16) -> bool {
    if new > old {
        (new - old) <= 32768
    } else {
        (old - new) > 32768
    }
}

fn smoothed_average(curr: f32, b: Duration) -> f32 {
    let av = (b.as_secs() as f32) * 1000.0 + b.subsec_millis() as f32;
    (curr - (curr - av) * 0.1).max(0.0)
}
