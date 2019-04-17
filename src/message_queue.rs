use std::cmp;
use std::collections::{BinaryHeap, HashMap};
use std::time::{Duration, Instant};

const MESSAGE_HEADER_LENGTH: usize = 4;
const BUFFER_SIZE: usize = 256;

#[derive(Eq, PartialEq, Clone)]
struct Message {
    id: u16,
    size: u16,
    data: Vec<u8>,
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        other.id.cmp(&self.id)
    }
}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct MessageQueue {
    sequence_local: u16,
    recent_acked: u16,
    sequence_remote: u16,
    awaiting_ack: HashMap<u16, Vec<u16>>,
    send_queue: Vec<Option<Message>>,
    recv_queue: BinaryHeap<Message>,
    recv: Vec<Vec<u8>>,
}

// message rtt
// adjust message sent based on rtt
impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            sequence_local: 0,
            recent_acked: 0,
            sequence_remote: 0,
            awaiting_ack: HashMap::new(),
            send_queue: vec![None; BUFFER_SIZE],
            recv_queue: BinaryHeap::new(),
            recv: Vec::new(),
        }
    }

    // Sending -- Queue message -> get to send -> acknowledge pack id when acked
    pub fn queue_message(&mut self, message: &[u8]) {
        let new_message = Message {
            id: self.sequence_local,
            size: message.len() as u16,
            data: message.to_vec(),
        };
        self.send_queue[self.sequence_local as usize % BUFFER_SIZE] = Some(new_message);
        self.sequence_local = self.sequence_local.wrapping_add(1);
    }

    pub fn send_next(&mut self, sequence: u16, amt: u16) -> Vec<u8> {
        let mut data = Vec::new();
        let mut ack_ids = Vec::new();
        let mut written = 0;
        let start = self.recent_acked as usize;
        let end = self.sequence_local as usize;

        for index in start..end {
            let normlz = index % BUFFER_SIZE;
            if let Some(message) = &mut self.send_queue[normlz] {
                written += message.size;
                if written < amt {
                    data.append(&mut message_into_vec(&message));
                    ack_ids.push(message.id);
                }
            }
        }
        self.awaiting_ack.insert(sequence, ack_ids);
        data
    }

    pub fn acknowledge(&mut self, pid: u16) {
        if let Some(ids) = self.awaiting_ack.get(&pid) {
            for id in ids.iter() {
                let index = *id as usize % BUFFER_SIZE;
                if let Some(msg) = &mut self.send_queue[index] {
                    if self.recent_acked < msg.id {
                        self.recent_acked = msg.id;
                    }
                    self.send_queue[index] = None;
                }
            }
            self.awaiting_ack.remove(&pid);
        }
    }

    // Receiving -- receive message internally -> recv all queued messages
    pub fn recv_next_all(&mut self) -> Vec<Vec<u8>> {
        let mut r = Vec::new();
        r.append(&mut self.recv);
        r
    }

    // Should return Result<usize, ParseErr> where usize is no. messages or something
    pub fn recv_messages(&mut self, slice: &[u8]) {
        let len = slice.len();
        let mut index = 0;
        while index < len && len - index >= MESSAGE_HEADER_LENGTH {
            // extract headers
            let id = ((slice[index] as u16) << 8) | slice[index + 1] as u16;
            let size = ((slice[index + 2] as u16) << 8) | slice[index + 3] as u16;
            index += MESSAGE_HEADER_LENGTH;

            // Extract data based on headers.
            // Currently will panic if headers are incorrect.
            let new_index = index + size as usize;
            let data = slice[index..new_index].to_vec();

            if id == self.sequence_remote {
                self.recv.push(data);
                self.sequence_remote = self.sequence_remote.wrapping_add(1);
            } else {
                self.recv_queue.push(Message { id, size, data });
            }
            index = new_index;
        }

        // move queued ordered messages from queue to recv if prev have been received
        let mut expected = true;
        while expected {
            expected = if let Some(msg) = self.recv_queue.peek() {
                msg.id == self.sequence_remote
            } else {
                false
            };

            if expected {
                let msg = self.recv_queue.pop().unwrap();
                self.recv.push(msg.data);

                self.sequence_remote = self.sequence_remote.wrapping_add(1);
            }
        }
    }
}

fn message_into_vec(message: &Message) -> Vec<u8> {
    let mut vec = Vec::new();

    vec.push((message.id >> 8) as u8);
    vec.push(message.id as u8);

    vec.push((message.size >> 8) as u8);
    vec.push(message.size as u8);

    vec.append(&mut message.data.clone());

    vec
}
