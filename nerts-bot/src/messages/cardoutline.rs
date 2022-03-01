use super::io::reader::{Deserialize, MessageReader};

#[derive(Debug)]
pub struct CardOutlineMessage {
    pub x: i16,
    pub y: i16,
}

impl Deserialize for CardOutlineMessage {
    fn deserialize(r: &mut MessageReader) -> Self {
        CardOutlineMessage {
            x: r.read(),
            y: r.read(),
        }
    }
}
