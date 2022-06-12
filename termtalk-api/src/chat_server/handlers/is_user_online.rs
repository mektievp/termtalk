use crate::chat_server::chat_server::ChatServer;
use actix::prelude::*;

pub struct IsUserOnline {
    pub username: String,
}

impl actix::Message for IsUserOnline {
    type Result = bool;
}
impl Handler<IsUserOnline> for ChatServer {
    type Result = bool;

    fn handle(&mut self, msg: IsUserOnline, _ctx: &mut Context<Self>) -> bool {
        let redis = self.redis.clone();
        redis.users_online_set.user_online(&msg.username)
    }
}
