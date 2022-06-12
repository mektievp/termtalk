use super::users::UsersElasticStore;

#[derive(Clone, Debug)]
pub struct ElasticStore {
    pub users: UsersElasticStore,
}

impl ElasticStore {
    pub fn new(elastic_client: elasticsearch::Elasticsearch) -> ElasticStore {
        ElasticStore {
            users: UsersElasticStore::new(elastic_client.clone()),
        }
    }
}
