use std::cmp;
use std::collections::{BinaryHeap, VecDeque};

const MESSAGE_HEADER_LENGTH: usize = 4;
const BUFFER_SIZE: usize = 256;

#[derive(Eq, PartialEq)]
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
    sequence_remote: u16,
    send_queue: VecDeque<Message>,
    recv_queue: BinaryHeap<Message>,
    recv: Vec<Vec<u8>>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            sequence_local: 0,
            sequence_remote: 0,
            send_queue: VecDeque::new(),
            recv_queue: BinaryHeap::new(),
            recv: Vec::new(),
        }
    }

    pub fn queue_message(&mut self, message: Vec<u8>) {
        let new_message = Message {
            id: self.sequence_local,
            size: message.len() as u16,
            data: message,
        };
    }

    pub fn recv_next(&mut self) -> Option<Vec<Vec<u8>>> {
        if self.recv.len() < 1 {
            return None;
        }
        let next = Some(self.recv);
        self.recv = Vec::new();
        next
    }

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

            self.recv_queue.push(Message { id, size, data });

            index = new_index;
        }

        // move ordered messages from queue to recv
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

                self.sequence_remote.wrapping_add(1);
            }
        }
    }
}
