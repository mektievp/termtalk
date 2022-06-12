use super::store::{RedisSet, RedisSetFns, RedisUtilityFunc};

static USERS_ONLINE: &str = "USERS_ONLINE";

#[derive(Clone, Debug)]
pub struct UsersOnlineSet {
    redis: redis::Client,
}

impl UsersOnlineSet {
    pub fn new(redis_client: redis::Client) -> UsersOnlineSet {
        Self {
            redis: redis_client.clone(),
        }
    }
}

impl RedisUtilityFunc for UsersOnlineSet {
    fn get_redis_attr(&self) -> redis::Client {
        self.redis.clone()
    }
}

impl RedisSet for UsersOnlineSet {
    fn set_name() -> String {
        USERS_ONLINE.to_owned()
    }
}

impl UsersOnlineSet {
    pub fn add_to_users_online_set(&self, val: &str) -> bool {
        self.sadd(USERS_ONLINE, val)
    }

    pub fn remove_from_users_online_set(&self, val: &str) -> bool {
        self.srem(USERS_ONLINE, val)
    }

    pub fn user_online(&self, val: &str) -> bool {
        self.sismember(USERS_ONLINE, &val)
    }

    pub fn users_online(&self) -> Vec<String> {
        self.smembers(USERS_ONLINE)
    }
}
