use super::io::writer::{MessageWriter, Serialize};

#[derive(Debug)]
pub struct ClientMessage {
    pub x: i16,
    pub y: i16,
    pub left_click: bool,
    pub right_click: bool,
    pub make_ready: bool,
    pub draw: bool,
    pub card_back: u8,
    pub card_color: u8,
    pub send_key_frame: bool,
}

impl Serialize for ClientMessage {
    fn serialize(&self, w: &mut MessageWriter) {
        w.write_i16(self.x);
        w.write_i16(self.y);
        w.write_bool(self.left_click);
        w.write_bool(self.right_click);
        w.write_bool(self.make_ready);
        w.write_bool(self.draw);
        w.write_u8(self.card_back);
        w.write_u8(self.card_color);
        w.write_bool(self.send_key_frame);
    }
}
