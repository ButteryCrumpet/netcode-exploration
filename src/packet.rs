
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

    pub fn from_vec(vec: &Vec<u8>) -> Result<Self, &str> {

        if vec.len() < 8 {
            return Err("vec is too short noob")
        }

        let sequence = ((vec[0] as u16) << 8) | vec[1] as u16;
        let ack = ((vec[2] as u16) << 8) | vec[3] as u16;
        
        let bits = ((vec[4] as u32) << 24)
            | ((vec[5] as u32) << 16)
            | ((vec[6] as u32) << 8)
            | vec[7] as u32;

        let mut acks: Vec<u16> = Vec::new();
        for i in 0..32 {
            if bits & (1 << i) != 0 {
                acks.push(ack - i as u16)
            };
        }

        Ok(Packet { sequence, ack, acks })
    }

    pub fn into_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();

        // Push sent sequence number
        vec.push((self.sequence >> 8) as u8);
        vec.push(self.sequence as u8);

        // Push received sequence number
        vec.push((self.ack >> 8) as u8);
        vec.push((self.ack) as u8);

        // Set bits for each sequence to ack
        let mut ack_bits: u32 = 0;
        for seq in self.acks.iter().take(32) {
            ack_bits |= 1 << ((self.ack as u32) - (*seq as u32))
        }

        // Push bitset
        vec.push((ack_bits >> 24) as u8);
        vec.push((ack_bits >> 16) as u8);
        vec.push((ack_bits >> 8) as u8);
        vec.push(ack_bits as u8);

        vec
    }
}


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let packet = Packet::new(5, 7, vec![7,5,3,2,1]);
        let vec =  packet.into_vec();
        let new = Packet::from_vec(&vec).unwrap();
        assert_eq!(packet, new);
    }
   
}