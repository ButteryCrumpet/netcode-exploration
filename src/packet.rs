
pub struct Packet {
    pub sequence: u16,
    pub ack: u16,
    pub ack_bits: [bool; 32]
}

impl Packet {

    pub fn new(sequence: u16, ack: u16, ack_bits: [bool; 32]) -> Self {
        Packet { sequence, ack, ack_bits }
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

        let mut ack_bits: [bool; 32] = [false; 32];
        for i in 0..31 {
            ack_bits[0] = bit_at(bits, i);
        }

        Ok(Packet {
            sequence,
            ack,
            ack_bits,
        })
    }

    pub fn into_vec(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.push((self.sequence >> 8) as u8);
        vec.push(self.sequence as u8);
        vec.push((self.ack >> 8) as u8);
        vec.push((self.ack) as u8);

        let acks = u32_to_u8_slice(bits_to_u32(self.ack_bits));
        for i in 0..3 {
            vec.push(acks[i]);
        }

        vec
    }
}

fn bit_at(bits: u32, n: u8) -> bool {
    if n < 32 {
        return bits & (1 << n) != 0
    }
    panic!("n must be smaller than 32")
}

fn bits_to_u32(bits: [bool; 32]) -> u32 {
    let mut int: u32 = 0;
    for i in 0..31 {
        int |= 1 << bits[i] as u32;
    }
    int
}

fn u32_to_u8_slice(int: u32) -> [u8;4] {
    let n1 = (int >> 24) as u8;
    let n2 = (int >> 16) as u8;
    let n3 = (int >> 8) as u8;
    let n4 = int as u8;
    [n1, n2, n3, n4]
}