use crate::chat_server::chat_server::{ChatServer, ChatSessionState, ChatType, Message};
use actix::prelude::*;

#[derive(Message)]
#[rtype(bool)]
pub struct Connect {
    pub username: String,
    pub channel_name: String,
    pub addr: Recipient<Message>,
    pub chat_type: ChatType,
}

impl Handler<Connect> for ChatServer {
    type Result = bool;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        log::debug!("{} has connected to the server", msg.username);
        let user_already_logged_in: bool = self.redis.users_online_set.user_online(&msg.username);
        if user_already_logged_in {
            return false;
        }

        self.redis
            .users_online_set
            .add_to_users_online_set(&msg.username);
        self.sessions.insert(
            msg.username.clone(),
            ChatSessionState {
                username: msg.username.clone(),
                channel_name: msg.channel_name.clone(),
                chat_type: msg.chat_type.clone(),
                addr: msg.addr,
            },
        );
        true
    }
}
