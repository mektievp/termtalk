use elasticsearch::indices::IndicesCreateParts;
use elasticsearch::Elasticsearch;
use log::info;
use serde_json::{Value};
use std::fs;

fn elastic_client() -> elasticsearch::Elasticsearch {
    Elasticsearch::default()
}

async fn create_index(index_name: &str, file_path_str: &str) {
    let contents: String = match fs::read_to_string(file_path_str) {
        Ok(val) => val,
        Err(error) => panic!("Something weng wrong while attempting to read '{}' to string: {}", file_path_str, error),
    };
    let contents_serialized: Value = match serde_json::from_str(&contents) {
        Ok(val) => val,
        Err(error) => panic!("Something went wrong while attempting to serialize contents for '{}': {}", index_name, error),
    };

    let client = elastic_client();
    let create_index_resp = match client
        .indices()
        .create(IndicesCreateParts::Index(index_name))
        .body(contents_serialized)
        .send()
        .await
    {
        Ok(val) => val,
        Err(error) => panic!("Something went wrong while creating an index: {}", error),
    };
    info!("Creating index '{}'", index_name);
    let response_body: String = match create_index_resp.json::<Value>().await {
        Ok(val) => val.to_string(),
        Err(error) => panic!(
            "Something went wrong while creating index '{}': {}",
            index_name, error
        ),
    };
    info!(
        "Create index response for '{}': {}",
        index_name, response_body
    );
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let paths = match fs::read_dir("./index_mappings") {
        Ok(val) => val,
        Err(error) => panic!(
            "Something went wrong while reading from the 'index_mappings' dir: {}",
            error
        ),
    };
    for path in paths {
        let path_result: std::fs::DirEntry = match path {
            Ok(val) => val,
            Err(error) => panic!(
                "Something went wrong while attempting to unwrap the path result: {}",
                error
            ),
        };
        let file_path_str: String = match path_result.path().into_os_string().into_string() {
            Ok(val) => val,
            Err(error) => panic!("Something went wrong while attempting to retrieve file_path_str from path_result: {:?}", error),
        };
        let path_result_file_name = path_result.file_name();
        let path_result_str: String = match path_result_file_name.into_string() {
            Ok(val) => val,
            Err(error) => panic!("Something went wrong while attempting to convert path_result_file_name into a string: {:?}", error),
        };
        let mut path_result_str_split = path_result_str.split(".");
        let index_name: String = match path_result_str_split.nth(0) {
            Some(val) => val.to_string(),
            None => continue,
        };
        create_index(&index_name, &file_path_str).await;
    }
}
