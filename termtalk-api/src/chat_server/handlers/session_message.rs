use crate::chat_server::chat_server::{ChatServer, ChatType, MessageType, QueueMessage};
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SessionMessage {
    pub username: String,
    pub msg: String,
    pub channel_name: String,
    pub chat_type: ChatType,
    pub msg_type: MessageType,
}

impl Handler<SessionMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SessionMessage, _: &mut Context<Self>) {
        let chat_message = QueueMessage {
            sender: msg.username,
            msg: msg.msg,
            chat_type: msg.chat_type,
            msg_type: msg.msg_type,
            recipient: msg.channel_name,
        };
        self.redis
            .publish_chat_messages
            .publish_to_channel(chat_message);
    }
}
