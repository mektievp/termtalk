use super::store::{RedisPubSub, RedisPubSubFns, RedisUtilityFunc};
use crate::chat_server::chat_server::QueueMessage;

pub static CHAT_MESSAGES: &str = "CHAT_MESSAGES";

#[derive(Clone, Debug)]
pub struct PubSubChatMessages {
    redis: redis::Client,
}

impl PubSubChatMessages {
    pub fn new(redis_client: redis::Client) -> PubSubChatMessages {
        Self {
            redis: redis_client.clone(),
        }
    }
}

impl RedisUtilityFunc for PubSubChatMessages {
    fn get_redis_attr(&self) -> redis::Client {
        self.redis.clone()
    }
}

impl RedisPubSub for PubSubChatMessages {
    fn pubsub_name() -> String {
        CHAT_MESSAGES.to_owned()
    }
}

impl PubSubChatMessages {
    pub fn publish_to_channel(&self, chat_message: QueueMessage) -> bool {
        let chat_message_str = serde_json::to_string(&chat_message).unwrap();
        self.publish(CHAT_MESSAGES, &chat_message_str)
    }
}
