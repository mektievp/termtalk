use crate::chat_server::chat_server::ChatServer;
use actix::prelude::*;

pub struct ListRooms;

impl actix::Message for ListRooms {
    type Result = Vec<String>;
}

impl Handler<ListRooms> for ChatServer {
    type Result = MessageResult<ListRooms>;

    fn handle(&mut self, _: ListRooms, _: &mut Context<Self>) -> Self::Result {
        let list_rooms: Vec<String> = self.redis.rooms_hash_map.list_rooms();

        MessageResult(list_rooms)
    }
}
