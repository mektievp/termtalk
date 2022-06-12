use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TermQuery<T> {
    pub hits: TermQueryHits<T>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TermQueryHits<T> {
    pub hits: Vec<DocumentMetadata<T>>,
    pub total: TotalHits,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TotalHits {
    pub value: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DocumentMetadata<T> {
    pub _id: String,
    _index: String,
    pub _source: T,
}
