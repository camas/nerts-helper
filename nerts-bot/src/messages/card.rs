use super::io::reader::{Deserialize, MessageReader};

#[derive(Debug)]
pub struct CardMessage {
    pub x: i16,
    pub y: i16,
    pub data: u8,
    pub flags: u8,
    pub height: u8,
    pub holder: u8,
}

impl Deserialize for CardMessage {
    fn deserialize(r: &mut MessageReader) -> Self {
        CardMessage {
            x: r.read(),
            y: r.read(),
            data: r.read(),
            flags: r.read(),
            height: r.read(),
            holder: r.read(),
        }
    }
}

// #[derive(Debug)]
// pub struct CardFlags {
//     pub flags: u8,
// }

// impl CardFlags {
//     pub fn is_face_up(&self) -> bool {
//         self.flags & 0x01 != 0
//     }

//     pub fn is_flipped(&self) -> bool {
//         self.flags & 0x02 != 0
//     }

//     pub fn is_in_nerts_pile(&self) -> bool {
//         self.flags & 0x04 != 0
//     }

//     pub fn is_disable_foundation(&self) -> bool {
//         self.flags & 0x08 != 0
//     }

//     pub fn is_disable_personal(&self) -> bool {
//         self.flags & 0x10 != 0
//     }
// }
