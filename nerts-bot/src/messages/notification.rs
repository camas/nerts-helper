use super::io::reader::{Deserialize, MessageReader};

#[derive(Debug)]
pub struct NotificationMessage {
    pub player_id: u64,
    pub notification_type: u8,
}

impl Deserialize for NotificationMessage {
    fn deserialize(r: &mut MessageReader) -> Self {
        NotificationMessage {
            player_id: r.read(),
            notification_type: r.read(),
        }
    }
}
