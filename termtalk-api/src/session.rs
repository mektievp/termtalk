use std::time::{Duration, Instant};

use crate::chat_server::chat_server::{ChatServer, ChatType, Message, MessageType};
use crate::chat_server::handlers::{
    connect::Connect, debug_server::DebugServer, disconnect::Disconnect,
    is_user_online::IsUserOnline, join_direct::JoinDirect, join_room::JoinRoom,
    list_rooms::ListRooms, list_users_in_room::ListUsersInRoom, list_users_online::ListUsersOnline,
    session_message::SessionMessage, update_session_status::UpdateSessionStatus,
};
use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use actix_web_actors::ws::{CloseCode, CloseReason};
use serde_json::json;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    pub username: String,
    pub hb: Instant,
    pub addr: web::Data<Addr<ChatServer>>,
    pub channel_name: String,
    pub valid_connection: bool,
    pub chat_type: ChatType,
}

impl WsChatSession {
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");

                act.addr.do_send(Disconnect {
                    username: act.username.clone(),
                });

                ctx.stop();

                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();
        self.addr
            .send(Connect {
                username: self.username.clone(),
                channel_name: self.channel_name.clone(),
                chat_type: self.chat_type.clone(),
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => {
                        if res == false {
                            let close_reason = CloseReason {
                                code: CloseCode::Invalid,
                                description: Some(
                                    "user is already connected from another client".to_string(),
                                ),
                            };
                            ctx.close(Some(close_reason));
                            ctx.stop();
                        } else {
                            act.valid_connection = true;
                            act.hb(ctx)
                        }
                    }
                    _ => {
                        ctx.stop();
                    }
                }
                return fut::ready(());
            })
            .wait(ctx);

        self.addr.do_send(JoinRoom {
            username: self.username.clone(),
            channel_name: self.channel_name.clone(),
            chat_type: ChatType::Room,
            previous_channel_name: self.channel_name.clone(),
            previous_chat_type: ChatType::NoPreviousChatType,
        });
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        if self.valid_connection {
            self.addr.do_send(Disconnect {
                username: self.username.clone(),
            });
        }
        Running::Stop
    }
}

impl Handler<Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };

        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                let m = text.trim();
                if m.starts_with('/') {
                    let v: Vec<&str> = m.splitn(3, ' ').collect();
                    match v[0] {
                        "/debug" => log::info!("self: {:?}", self),
                        "/debug_server" => {
                            self.addr.do_send(DebugServer);
                        },
                        "/whoami" => {
                            let whereami = match &self.chat_type {
                                ChatType::Direct => {
                                    let users_iterator = self.channel_name.split("_");
                                    let mut direct_user = String::from("");
                                    for user in users_iterator {
                                        if user != self.username {
                                            direct_user = user.to_owned();
                                            break;
                                        }
                                    };
                                    format!("directly messaging {}", direct_user)
                                },
                                ChatType::Room => {
                                    format!("in room {}", self.channel_name)
                                },
                                _ => String::from(""),
                            };
                            let msg_struct = Message {text: format!("You are {} and you are currently {}", self.username, whereami), color: "green".to_string()};
                            let msg = serde_json::to_string(&msg_struct).unwrap();
                            ctx.text(msg);
                        }
                        "/o" | "/online" => {

                            self.addr
                                .send(ListUsersOnline)
                                .into_actor(self)
                                .then(|res, _, ctx| {
                                    match res {
                                        Ok(connected_users) => {

                                            ctx.text(json!({"text": "Users currently online", "color": "green"}).to_string());
                                            for user in connected_users {
                                                let msg_struct = Message {text: user.to_owned(), color: "green".to_string()};
                                                let msg = serde_json::to_string(&msg_struct).unwrap();
                                                ctx.text(msg);
                                            }
                                        },
                                        _ => log::debug!("Something went wrong while fetching online users"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx)
                        }
                        "/j" | "/join" => {
                            if v.len() == 2 {
                                let channel_name = v[1].to_owned();
                                if self.chat_type == ChatType::Room && self.channel_name == channel_name {
                                    let msg = Message{
                                        text: format!("You are already in room {}", channel_name),
                                        color: "green".to_owned(),
                                    };
                                    ctx.text(serde_json::to_string(&msg).unwrap());
                                    return
                                }
                                self.addr.do_send(JoinRoom {
                                    username: self.username.clone(),
                                    channel_name: channel_name.clone(),
                                    chat_type: ChatType::Room,
                                    previous_channel_name: self.channel_name.clone(),
                                    previous_chat_type: self.chat_type.clone(),
                                });
                                self.addr
                                    .do_send(UpdateSessionStatus{
                                        username: self.username.clone(),
                                        channel_name: channel_name.clone(),
                                        chat_type: ChatType::Room,
                                    });
                                self.channel_name =  channel_name.clone();
                                self.chat_type = ChatType::Room;

                            } else {
                                ctx.text("!!! room name is required");
                            }
                        }
                        "/r" | "/rooms" => {
                            self.addr
                                .send(ListRooms)
                                .into_actor(self)
                                .then(|res, _act, ctx| {
                                    match res {
                                        Ok(list_rooms) => {
                                            ctx.text(json!({"text": "List of Existing Rooms", "color": "green"}).to_string());
                                            for room in list_rooms {
                                                let msg_struct = Message {text: room.to_owned(), color: "green".to_string()};
                                                let msg = serde_json::to_string(&msg_struct).unwrap();
                                                ctx.text(msg);
                                            }
                                        },
                                        _ => log::debug!("Something went wrong while list of existing rooms"),
                                    }
                                    fut::ready(())
                                })
                                .wait(ctx);
                        }
                        "/h" | "/here" => {
                            self.addr
                            .send(ListUsersInRoom {
                                room: self.channel_name.clone(),
                            })
                            .into_actor(self)
                            .then(|res, _act, ctx| {
                                match res {
                                    Ok(rooms_users) => {
                                        ctx.text(json!({"text": "List of who is here", "color": "green"}).to_string());
                                        for user in rooms_users {
                                            let msg_struct = Message {text: user.to_owned(), color: "green".to_string()};
                                            let msg = serde_json::to_string(&msg_struct).unwrap();
                                            ctx.text(msg);
                                        }
                                    },
                                    _ => log::debug!("Something went wrong while list of existing rooms"),
                                }
                                fut::ready(())
                            })
                            .wait(ctx);

                        }
                        "/w" | "/whisper" => {
                            if v.len() == 3 {

                                let recipient = v[1].to_owned();
                                if recipient == self.username {
                                    let msg = Message{
                                        text: "You cannot whisper to yourself".to_owned(),
                                        color: "green".to_owned(),
                                    };
                                    ctx.text(serde_json::to_string(&msg).unwrap());
                                    return
                                };
                                let text = v[2].to_owned();

                                self.addr
                                    .send(IsUserOnline{username: recipient.to_owned()})
                                    .into_actor(self)
                                    .then(move |res, act, ctx| {
                                        match res {
                                            Ok(user_exists) => {
                                                if user_exists {
                                                    act.addr.do_send(SessionMessage {
                                                        username: act.username.clone(),
                                                        msg: text,
                                                        channel_name: recipient.clone(),
                                                        chat_type: ChatType::Whisper,
                                                        msg_type: MessageType::Whisper,
                                                    })

                                                } else {
                                                    let mut msg = Message{
                                                        text: "".to_owned(),
                                                        color: "green".to_owned(),
                                                    };
                                                    msg.text = format!("User {} is not currently online. Try again", &recipient).to_owned();
                                                    ctx.text(serde_json::to_string(&msg).unwrap());
                                                }
                                            },
                                            e => log::debug!("Encountered an error while trying to determine if user {} exists. Error: {:?}", &recipient, e),
                                        }

                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            } else {
                                let mut msg = Message{
                                    text: "".to_owned(),
                                    color: "green".to_owned(),
                                };
                                msg.text = "/w | /whisper command requires that you pass in a username as an argument followed by a message. Try again".to_owned();
                                ctx.text(serde_json::to_string(&msg).unwrap());
                            }

                        }
                        "/d" | "/direct" => {
                            let mut msg = Message{
                                text: "".to_owned(),
                                color: "green".to_owned(),
                            };
                            if v.len() == 2 {

                                let recipient = v[1].to_owned();
                                if recipient == self.username {
                                    let msg = Message{
                                        text: "You cannot direct chat yourself".to_owned(),
                                        color: "green".to_owned(),
                                    };
                                    ctx.text(serde_json::to_string(&msg).unwrap());
                                    return
                                }
                                self.addr
                                    .send(IsUserOnline{username: recipient.to_owned()})
                                    .into_actor(self)
                                    .then(move |res, act, ctx| {

                                        match res {
                                            Ok(user_online) => {
                                                if user_online {
                                                    let mut user_vec = vec![act.username.clone(), recipient.clone()];
                                                    user_vec.sort();

                                                    let channel_name = user_vec.join("_");
                                                    if channel_name == act.channel_name {
                                                        let msg = Message{
                                                            text: format!("You are already direct messaging user {}", recipient),
                                                            color: "green".to_owned(),
                                                        };
                                                        ctx.text(serde_json::to_string(&msg).unwrap());
                                                        return fut::ready(());
                                                    }

                                                    act.addr
                                                    .do_send(JoinDirect{
                                                        recipient: recipient.to_owned(),
                                                        sender: act.username.clone(),
                                                        channel_name: channel_name.clone(),
                                                        previous_channel_name: act.channel_name.clone(),
                                                        previous_chat_type: act.chat_type.clone(),
                                                    });
                                                    act.addr
                                                        .do_send(UpdateSessionStatus{
                                                            username: act.username.clone(),
                                                            channel_name: channel_name.clone(),
                                                            chat_type: ChatType::Direct,
                                                        });

                                                    act.channel_name = channel_name.clone();
                                                    act.chat_type = ChatType::Direct;
                                                } else {
                                                    msg.text = format!("User {} is not currently online. Try again", &recipient).to_owned();
                                                    ctx.text(serde_json::to_string(&msg).unwrap());
                                                }

                                            },
                                            e => log::debug!("Encountered an error while trying to determine if user {} exists. Error: {:?}", &recipient, e),
                                        }
                                        fut::ready(())
                                    })
                                    .wait(ctx);
                            } else {
                                msg.text = "/d | /direct command requires that you pass in a username as an argument. Try again".to_owned();
                                ctx.text(serde_json::to_string(&msg).unwrap());
                            }
                        }
                        _ => ctx.text(format!("!!! unknown command: {:?}", m)),
                    }
                } else {
                    let msg_type = match self.chat_type {
                        ChatType::Room => MessageType::Room,
                        ChatType::Direct => MessageType::Direct,
                        _ => MessageType::Room,
                    };
                    self.addr.do_send(SessionMessage {
                        username: self.username.clone(),
                        msg: m.to_owned(),
                        channel_name: self.channel_name.clone(),
                        chat_type: self.chat_type.clone(),
                        msg_type: msg_type,
                    })
                }
            }
            ws::Message::Binary(_) => println!("Unexpected binary"),
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            ws::Message::Nop => (),
        }
    }
}
