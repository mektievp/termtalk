use crate::chat_server::chat_server::ChatServer;
use actix::prelude::*;

pub struct ListUsersOnline;

impl actix::Message for ListUsersOnline {
    type Result = Vec<String>;
}

impl Handler<ListUsersOnline> for ChatServer {
    type Result = MessageResult<ListUsersOnline>;

    fn handle(&mut self, _: ListUsersOnline, _: &mut Context<Self>) -> Self::Result {
        let users_online = self.redis.users_online_set.users_online();
        MessageResult(users_online)
    }
}
