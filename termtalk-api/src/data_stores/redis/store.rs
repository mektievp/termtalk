use super::{
    publish_chat_messages::PubSubChatMessages, rooms_hash_map::RoomsHashMap,
    rooms_online_users_set::RoomsOnlineUsersSet, users_online_set::UsersOnlineSet,
};
use redis::Commands;

#[derive(Clone, Debug)]
pub struct RedisStore {
    pub redis: redis::Client,
    pub rooms_hash_map: RoomsHashMap,
    pub users_online_set: UsersOnlineSet,
    pub publish_chat_messages: PubSubChatMessages,
    pub rooms_online_users_set: RoomsOnlineUsersSet,
}

impl RedisStore {
    pub fn new(redis_client: redis::Client) -> RedisStore {
        RedisStore {
            redis: redis_client.clone(),
            rooms_hash_map: RoomsHashMap::new(redis_client.clone()),
            users_online_set: UsersOnlineSet::new(redis_client.clone()),
            publish_chat_messages: PubSubChatMessages::new(redis_client.clone()),
            rooms_online_users_set: RoomsOnlineUsersSet::new(redis_client.clone()),
        }
    }
}

pub trait RedisUtilityFunc {
    fn get_redis_attr(&self) -> redis::Client;
    fn get_connection(&self) -> redis::Connection {
        self.get_redis_attr().get_connection().unwrap()
    }
}

pub trait RedisHashMap {
    fn hash_map_name() -> String;
}

pub trait RedisHashMapFns {
    fn hget(&self, hash_map_name: &str, key: &str) -> Option<String>;
    fn hset(&self, hash_map_name: &str, key: &str, val: &str) -> Option<String>;
    fn hkeys(&self, hash_map_name: &str) -> Vec<String>;
    fn hvals(&self, hash_map_name: &str) -> Vec<String>;
}

impl<T> RedisHashMapFns for T
where
    T: RedisUtilityFunc + RedisHashMap,
{
    fn hget(&self, hash_map_name: &str, key: &str) -> Option<String> {
        self.get_connection().hget(hash_map_name, &key).unwrap()
    }

    fn hset(&self, hash_map_name: &str, key: &str, val: &str) -> Option<String> {
        self.get_connection()
            .hset(hash_map_name, &key, &val)
            .unwrap()
    }

    fn hkeys(&self, hash_map_name: &str) -> Vec<String> {
        self.get_connection().hkeys(hash_map_name).unwrap()
    }

    fn hvals(&self, hash_map_name: &str) -> Vec<String> {
        self.get_connection().hvals(hash_map_name).unwrap()
    }
}

pub trait RedisSet {
    fn set_name() -> String;
}

pub trait RedisSetFns {
    fn sadd(&self, set_name: &str, val: &str) -> bool;
    fn srem(&self, set_name: &str, val: &str) -> bool;
    fn sismember(&self, set_name: &str, val: &str) -> bool;
    fn smembers(&self, set_name: &str) -> Vec<String>;
}

impl<T> RedisSetFns for T
where
    T: RedisUtilityFunc + RedisSet,
{
    fn sadd(&self, set_name: &str, key: &str) -> bool {
        self.get_connection().sadd(set_name, &key).unwrap()
    }

    fn srem(&self, set_name: &str, val: &str) -> bool {
        self.get_connection().srem(set_name, &val).unwrap()
    }

    fn sismember(&self, set_name: &str, val: &str) -> bool {
        self.get_connection().sismember(set_name, &val).unwrap()
    }

    fn smembers(&self, set_name: &str) -> Vec<String> {
        self.get_connection().smembers(set_name).unwrap()
    }
}

pub trait RedisPubSub {
    fn pubsub_name() -> String;
}

pub trait RedisPubSubFns {
    fn publish(&self, channel_name: &str, val: &str) -> bool;
}

impl<T> RedisPubSubFns for T
where
    T: RedisUtilityFunc + RedisPubSub,
{
    fn publish(&self, channel_name: &str, val: &str) -> bool {
        self.get_connection().publish(channel_name, &val).unwrap()
    }
}
