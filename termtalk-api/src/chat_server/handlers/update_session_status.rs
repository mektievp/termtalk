use crate::chat_server::chat_server::{ChatServer, ChatSessionState, ChatType};
use actix::prelude::*;

pub struct UpdateSessionStatus {
    pub username: String,
    pub channel_name: String,
    pub chat_type: ChatType,
}

impl actix::Message for UpdateSessionStatus {
    type Result = UpdateSessionStatus;
}

impl Handler<UpdateSessionStatus> for ChatServer {
    type Result = MessageResult<UpdateSessionStatus>;

    fn handle(&mut self, msg: UpdateSessionStatus, _: &mut Context<Self>) -> Self::Result {
        let mut user_session: &mut ChatSessionState = self.sessions.get_mut(&msg.username).unwrap();
        user_session.channel_name = msg.channel_name.clone();
        user_session.chat_type = msg.chat_type.clone();

        MessageResult(msg)
    }
}
