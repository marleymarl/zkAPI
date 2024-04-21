use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use hex::encode as hex_encode;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiResponse {
    pub some_response_param: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct VerifiedResponse {
    pub response: ApiResponse,
    pub request_hash: String,
}

pub fn generate_request_hash(url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    hex_encode(hasher.finalize())
}

pub fn fetch_data(url: &str) -> Result<VerifiedResponse, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let response = client.get(url).send()?;
    let api_response: ApiResponse = response.json()?;
    let request_hash = generate_request_hash(url);
    Ok(VerifiedResponse {
        response: api_response,
        request_hash,
    })
}

pub fn verify_response(received_hash: &str, url: &str) -> bool {
    let expected_hash = generate_request_hash(url);
    expected_hash == received_hash
}