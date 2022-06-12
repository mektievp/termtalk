use crate::chat_server::chat_server::ChatServer;
use actix::prelude::*;

pub struct DebugServer;

impl actix::Message for DebugServer {
    type Result = ();
}
impl Handler<DebugServer> for ChatServer {
    type Result = ();

    fn handle(&mut self, _msg: DebugServer, _ctx: &mut Context<Self>) -> Self::Result {
        log::info!("self.directs: {:?}\n", self.directs);
        log::info!("self.rooms: {:?}\n", self.rooms);
    }
}
