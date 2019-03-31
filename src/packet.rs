
#[derive(Debug, PartialEq)]
pub struct Packet {
    pub sequence: u16,
    pub ack: u16,
    pub acks: Vec<u16>
}

impl Packet {

    pub fn new(sequence: u16, ack: u16, acks: Vec<u16>) -> Self {
        Packet { sequence, ack, acks }
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, &str> {

        if slice.len() < 8 {
            return Err("slice is too short noob")
        }

        let sequence = ((slice[0] as u16) << 8) | slice[1] as u16;
        let ack = ((slice[2] as u16) << 8) | slice[3] as u16;

        let bits = ((slice[4] as u32) << 24)
            | ((slice[5] as u32) << 16)
            | ((slice[6] as u32) << 8)
            | slice[7] as u32;

        let mut acks: Vec<u16> = Vec::new();
        for i in 0..32 {
            if bits & (1 << i) != 0 {
                acks.push(ack.wrapping_sub(i))
            };
        }

        Ok(Packet { sequence, ack, acks })
    }

    pub fn into_slice(&self) -> Vec<u8> {
        let mut vec = Vec::new();

        // Push sent sequence number
        vec.push((self.sequence >> 8) as u8);
        vec.push(self.sequence as u8);

        // Push received sequence number
        vec.push((self.ack >> 8) as u8);
        vec.push((self.ack) as u8);

        // Set bits for each sequence to ack
        let mut ack_bits: u32 = 0;
        for seq in self.acks.iter() {
            ack_bits |= 1 << self.get_bit_index(*seq);
        }

        // Push bitset
        vec.push((ack_bits >> 24) as u8);
        vec.push((ack_bits >> 16) as u8);
        vec.push((ack_bits >> 8) as u8);
        vec.push(ack_bits as u8);

        vec
    }

    fn get_bit_index(&self, seq: u16) -> u32 {
        if seq > self.ack {
            (self.ack + (std::u16::MAX - seq)) as u32
        } else {
            (self.ack - seq) as u32
        }
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let packet = Packet::new(5, 7, vec![7,5,3,2,1]);
        let vec =  packet.into_slice();
        let new = Packet::from_slice(&vec).unwrap();
        assert_eq!(packet, new);
    }
   
}