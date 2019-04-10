// do_ack() ? just all in connection?
// Into MessageQueue with send_next(recent_acks?) -> Vec<u8>
// if recent ack > prev_recent ack, pop acked messages
// hand over next to send
// recv_messages(data) - sort out ordering, prep for recv next on order
// recv_next() -> Vec<MessageData(Vec<u8>)>

pub struct Message {
    id: u16,
    size: u16,
    data: Vec<u8>,
}

impl Message {
    pub fn new(id: u16, data: Vec<u8>) -> Self {
        let size = data.len() as u16;
        Message { id, data, size }
    }

    pub fn into_vec(mut self) -> Vec<u8> {
        let mut vec = Vec::new();

        vec.push((self.id >> 8) as u8);
        vec.push(self.id as u8);

        vec.push((self.size >> 8) as u8);
        vec.push(self.size as u8);

        vec.append(&mut self.data);

        vec
    }

    pub fn from_slice(slice: &[u8]) -> Result<Vec<Self>, &str> {
        let len = slice.len();

        if len < 8 {
            return Err("herp");
        }
        let mut vec = Vec::new();
        let mut index = 0;

        while index < len && len - index >= 4 {
            let id = ((slice[index] as u16) << 8) | slice[index + 1] as u16;
            let size = ((slice[index + 2] as u16) << 8) | slice[index + 3] as u16;

            if index + (size as usize) + 4 > len {
                return Err("derp");
            }
            let new_index = index + 4 + size as usize;
            let data = slice[index + 4..new_index].to_vec();

            vec.push(Message { id, size, data });
            index = new_index;
        }
        Ok(vec)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let m1 = "hi";
        let m2 = "ho";
        let mess1 = Message::new(1, Vec::from(m1));
        let mess2 = Message::new(1, Vec::from(m2));
        let mut mv = mess1.into_vec();
        mv.append(&mut mess2.into_vec());
        let des = Message::from_slice(&mv).unwrap();
        assert_eq!(std::str::from_utf8(&des[0].data).unwrap(), m1);
        assert_eq!(std::str::from_utf8(&des[1].data).unwrap(), m2);
    }

}
