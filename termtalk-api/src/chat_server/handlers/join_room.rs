use crate::chat_server::chat_server::{ChatServer, ChatType, MessageType, QueueMessage};
use actix::prelude::*;
use std::collections::HashSet;

#[derive(Message)]
#[rtype(result = "()")]
pub struct JoinRoom {
    pub username: String,
    pub channel_name: String,
    pub chat_type: ChatType,
    pub previous_channel_name: String,
    pub previous_chat_type: ChatType,
}

impl Handler<JoinRoom> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) {
        match msg.previous_chat_type {
            ChatType::Direct => {
                if let Some(direct_state) = self.directs.get_mut(&msg.previous_channel_name) {
                    direct_state.remove(&msg.username);
                }
                let chat_message = QueueMessage {
                    sender: msg.username.clone(),
                    msg: format!("User {} left direct chat", &msg.username),
                    chat_type: ChatType::Direct,
                    msg_type: MessageType::Server,
                    recipient: msg.previous_channel_name.clone(),
                };

                self.redis
                    .publish_chat_messages
                    .publish_to_channel(chat_message);
            }
            ChatType::Room => {
                if let Some(room_state) = self.rooms.get_mut(&msg.previous_channel_name) {
                    room_state.remove(&msg.username);
                }
                self.redis
                    .rooms_online_users_set
                    .remove_user_from_room_set(&msg.channel_name, &msg.username);

                let chat_message = QueueMessage {
                    sender: msg.username.clone(),
                    msg: format!(
                        "User {} disconnected from room {}",
                        &msg.username, &msg.previous_channel_name
                    ),
                    chat_type: ChatType::Room,
                    msg_type: MessageType::Server,
                    recipient: msg.previous_channel_name.clone(),
                };

                self.redis
                    .publish_chat_messages
                    .publish_to_channel(chat_message);
            }
            _ => {}
        };

        let mut new_channel_set = HashSet::new();
        new_channel_set.insert(msg.username.clone());

        self.rooms
            .entry(msg.channel_name.clone())
            .and_modify(|e| {
                e.insert(msg.username.clone());
            })
            .or_insert(new_channel_set);

        let chat_message = QueueMessage {
            sender: msg.username.clone(),
            msg: format!(
                "User {} connected to room {}",
                &msg.username, &msg.channel_name
            ),
            chat_type: ChatType::Room,
            msg_type: MessageType::Server,
            recipient: msg.channel_name.clone(),
        };

        self.redis
            .publish_chat_messages
            .publish_to_channel(chat_message);
        self.redis
            .rooms_online_users_set
            .add_user_to_room_set(&msg.channel_name, &msg.username);
    }
}
