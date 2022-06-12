use crate::data_stores::{elastic::store::ElasticStore, redis::store::RedisStore};
use actix::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct ChatServer {
    pub sessions: HashMap<String, ChatSessionState>,
    pub rooms: HashMap<String, HashSet<String>>,
    pub directs: HashMap<String, HashSet<String>>,
    pub redis: RedisStore,
    pub elastic: ElasticStore,
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl ChatServer {
    pub async fn new(redis: RedisStore, elastic: ElasticStore) -> ChatServer {
        let directs = HashMap::new();
        let rooms = HashMap::new();

        ChatServer {
            sessions: HashMap::new(),
            rooms,
            directs,
            redis,
            elastic,
        }
    }

    fn select_color(&self, msg_type: &MessageType) -> String {
        match msg_type {
            MessageType::Direct => "blue".to_owned(),
            MessageType::Room => "white".to_owned(),
            MessageType::Whisper => "pink".to_owned(),
            MessageType::Server => "green".to_owned(),
        }
    }

    pub fn send_message_to_room(
        &self,
        channel_name: &str,
        sender: &str,
        message: &str,
        msg_type: &MessageType,
    ) {
        if let Some(channel_state) = self.rooms.get(channel_name) {
            for username in channel_state {
                let user_session = self.sessions.get(username).unwrap();
                let mut formatted_msg = format!("{}: {}", &sender, &message);
                if *msg_type == MessageType::Server {
                    formatted_msg = message.to_owned();
                }

                if user_session.channel_name == channel_name {
                    let _ = user_session.addr.do_send(Message {
                        text: formatted_msg,
                        color: self.select_color(msg_type),
                    });
                }
            }
        }
    }

    pub fn send_message_to_direct(
        &self,
        channel_name: &str,
        sender: &str,
        message: &str,
        msg_type: &MessageType,
    ) {
        let mut recipient = String::from("");
        let channel_users = channel_name.split("_").collect::<Vec<&str>>();
        for user in channel_users {
            if user != sender {
                recipient = user.to_owned();
            }
        }

        if let Some(recipient_session_state) = self.sessions.get(&recipient) {
            let mut recipient_formatted_msg = String::from("");

            match msg_type {
                MessageType::Server => {
                    if recipient_session_state.channel_name != channel_name
                        && *sender != recipient_session_state.username
                    {
                        recipient_formatted_msg = format!("(direct) {}", &message);
                    } else {
                        recipient_formatted_msg = message.to_string();
                    }
                }
                MessageType::Direct => {
                    if recipient_session_state.channel_name != channel_name {
                        recipient_formatted_msg = format!("(direct) {}: {}", &sender, &message);
                    } else {
                        recipient_formatted_msg = format!("{}: {}", &sender, &message);
                    }
                }
                _ => {}
            };

            let _ = recipient_session_state.addr.do_send(Message {
                text: recipient_formatted_msg,
                color: self.select_color(msg_type),
            });
        }

        if let Some(sender_state) = self.sessions.get(sender) {
            let mut sender_formatted_msg = String::from("");

            match msg_type {
                MessageType::Server => {
                    sender_formatted_msg = message.to_string();
                }
                MessageType::Direct => sender_formatted_msg = format!("{}: {}", &sender, &message),
                _ => {}
            };
            let _ = sender_state.addr.do_send(Message {
                text: sender_formatted_msg,
                color: self.select_color(msg_type),
            });
        }
    }

    pub fn whisper_message_to_recipient(&self, recipient: &str, sender: &str, message: &str) {
        if let Some(recipient_session) = self.sessions.get(recipient) {
            let _ = recipient_session.addr.do_send(Message {
                text: format!("(whisper) {} {}", &sender, &message),
                color: "pink".to_owned(),
            });
        }
        if let Some(sender_session) = self.sessions.get(sender) {
            let _ = sender_session.addr.do_send(Message {
                text: format!("(whisper) {} {}", &sender, &message),
                color: "pink".to_owned(),
            });
        }
    }
}

#[derive(Message, Serialize)]
#[rtype(result = "()")]
pub struct Message {
    pub text: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
pub enum ChatType {
    Direct,
    Room,
    Whisper,
    NoPreviousChatType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq)]
pub enum MessageType {
    Direct,
    Room,
    Whisper,
    Server,
}

#[derive(Debug, Hash)]
pub struct ChatSessionState {
    pub username: String,
    pub channel_name: String,
    pub addr: Recipient<Message>,
    pub chat_type: ChatType,
}

impl PartialEq for ChatSessionState {
    fn eq(&self, other: &Self) -> bool {
        self.username == other.username
    }

    fn ne(&self, other: &Self) -> bool {
        self.username != other.username
    }
}

impl Eq for ChatSessionState {}

#[derive(Serialize, Deserialize)]
pub struct QueueMessage {
    pub sender: String,
    pub chat_type: ChatType,
    pub msg_type: MessageType,
    pub recipient: String,
    pub msg: String,
}
