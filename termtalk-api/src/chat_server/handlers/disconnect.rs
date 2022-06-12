use crate::chat_server::chat_server::{ChatServer, ChatType, MessageType, QueueMessage};
use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub username: String,
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.redis
            .users_online_set
            .remove_from_users_online_set(&msg.username);

        let removed_session = self.sessions.remove(&msg.username);
        if removed_session.is_some() {
            let inner_removed_session = removed_session.unwrap();
            match inner_removed_session.chat_type {
                ChatType::Direct => {
                    if let Some(direct_state) =
                        self.directs.get_mut(&inner_removed_session.channel_name)
                    {
                        direct_state.remove(&inner_removed_session.username);
                    }
                }
                ChatType::Room => {
                    if let Some(room_state) =
                        self.rooms.get_mut(&inner_removed_session.channel_name)
                    {
                        room_state.remove(&inner_removed_session.username);
                    }
                    self.redis.rooms_online_users_set.remove_user_from_room_set(
                        &inner_removed_session.channel_name,
                        &inner_removed_session.username,
                    );

                    let chat_message = QueueMessage {
                        sender: msg.username.clone(),
                        msg: format!(
                            "User {} disconnected from {} and is now offline",
                            &msg.username, &inner_removed_session.channel_name
                        ),
                        chat_type: ChatType::Room,
                        msg_type: MessageType::Server,
                        recipient: inner_removed_session.channel_name.clone(),
                    };
                    self.redis
                        .publish_chat_messages
                        .publish_to_channel(chat_message);
                }
                _ => {}
            };
        }
    }
}
