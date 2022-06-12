use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomDocument {
    pub name: String,
    pub deleted_at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateRoomResult {
    pub _id: String,
    _index: String,
    result: String,
}
