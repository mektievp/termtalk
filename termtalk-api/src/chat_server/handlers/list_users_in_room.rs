use crate::chat_server::chat_server::ChatServer;
use actix::prelude::*;

pub struct ListUsersInRoom {
    pub room: String,
}

impl actix::Message for ListUsersInRoom {
    type Result = Vec<String>;
}

impl Handler<ListUsersInRoom> for ChatServer {
    type Result = MessageResult<ListUsersInRoom>;

    fn handle(&mut self, msg: ListUsersInRoom, _: &mut Context<Self>) -> Self::Result {
        let users_in_room: Vec<String> = self
            .redis
            .rooms_online_users_set
            .list_users_in_room(&msg.room);

        MessageResult(users_in_room)
    }
}
