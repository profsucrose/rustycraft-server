use crate::lib::events::RustyCraftMessage;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RustyCraftEvent {
    sender: String,
    message: RustyCraftMessage
}

pub fn serialize_event(sender: String, message: RustyCraftMessage) -> String {
    serde_json::to_string(&RustyCraftEvent { sender, message }).unwrap()
}