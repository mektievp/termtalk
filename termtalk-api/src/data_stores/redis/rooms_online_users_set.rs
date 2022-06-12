use super::store::{RedisSet, RedisSetFns, RedisUtilityFunc};

pub static ROOMS_ONLINE_USERS_SET: &str = "_ROOM_ONLINE_USERS_SET";

#[derive(Clone, Debug)]
pub struct RoomsOnlineUsersSet {
    redis: redis::Client,
}

impl RoomsOnlineUsersSet {
    pub fn new(redis_client: redis::Client) -> RoomsOnlineUsersSet {
        Self {
            redis: redis_client.clone(),
        }
    }
}

impl RedisUtilityFunc for RoomsOnlineUsersSet {
    fn get_redis_attr(&self) -> redis::Client {
        self.redis.clone()
    }
}

impl RedisSet for RoomsOnlineUsersSet {
    fn set_name() -> String {
        ROOMS_ONLINE_USERS_SET.to_owned()
    }
}

impl RoomsOnlineUsersSet {
    pub fn add_user_to_room_set(&self, room: &str, val: &str) -> bool {
        self.sadd(&format!("{}{}", room, ROOMS_ONLINE_USERS_SET), val)
    }

    pub fn remove_user_from_room_set(&self, room: &str, user: &str) -> bool {
        self.srem(&format!("{}{}", room, ROOMS_ONLINE_USERS_SET), user)
    }

    pub fn list_users_in_room(&self, room: &str) -> Vec<String> {
        self.smembers(&format!("{}{}", room, ROOMS_ONLINE_USERS_SET))
    }
}
