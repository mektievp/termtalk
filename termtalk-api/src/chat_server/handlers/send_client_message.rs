use crate::chat_server::chat_server::{ChatServer, ChatType, MessageType};
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct SendClientMessage {
    pub sender: String,
    pub msg: String,
    pub chat_type: ChatType,
    pub msg_type: MessageType,
    pub recipient: String,
}

impl Handler<SendClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendClientMessage, _: &mut Context<Self>) {
        match msg.chat_type {
            ChatType::Direct => self.send_message_to_direct(
                &msg.recipient,
                &msg.sender,
                msg.msg.as_str(),
                &msg.msg_type,
            ),
            ChatType::Room => self.send_message_to_room(
                &msg.recipient,
                &msg.sender,
                msg.msg.as_str(),
                &msg.msg_type,
            ),
            ChatType::Whisper => {
                self.whisper_message_to_recipient(&msg.recipient, &msg.sender, msg.msg.as_str())
            }
            _ => {}
        };
    }
}
