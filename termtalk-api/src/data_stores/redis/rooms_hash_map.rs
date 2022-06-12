use super::store::{RedisHashMap, RedisHashMapFns, RedisUtilityFunc};

pub static ROOMS_HASH_MAP: &str = "ROOMS_HASH_MAP";

#[derive(Clone, Debug)]
pub struct RoomsHashMap {
    redis: redis::Client,
}

impl RoomsHashMap {
    pub fn new(redis_client: redis::Client) -> RoomsHashMap {
        Self {
            redis: redis_client.clone(),
        }
    }
}

impl RedisUtilityFunc for RoomsHashMap {
    fn get_redis_attr(&self) -> redis::Client {
        self.redis.clone()
    }
}

impl RedisHashMap for RoomsHashMap {
    fn hash_map_name() -> String {
        ROOMS_HASH_MAP.to_owned()
    }
}

impl RoomsHashMap {
    pub fn _get_room_guid_by_name(&self, room_name: &str) -> Option<String> {
        self.hget(ROOMS_HASH_MAP, room_name)
    }

    pub fn _set_room_guid_by_name(&self, room_guid: &str, room_name: &str) -> Option<String> {
        self.hset(ROOMS_HASH_MAP, room_guid, room_name)
    }

    pub fn list_rooms(&self) -> Vec<String> {
        self.hkeys(ROOMS_HASH_MAP)
    }
}
