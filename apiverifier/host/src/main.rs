use std::time::Duration;
use anyhow::Result;
use bonsai_sdk::alpha as bonsai_sdk;
use methods::{METHOD_ELF, METHOD_ID};
use risc0_zkvm::{compute_image_id, serde::to_vec, Receipt};
use apiverifier::{VerifiedResponse, verify_response};

fn run_bonsai(url: &str) -> Result<VerifiedResponse> {
    let client = bonsai_sdk::Client::from_env(risc0_zkvm::VERSION)?;
    let image_id = hex::encode(compute_image_id(METHOD_ELF)?);
    client.upload_img(&image_id, METHOD_ELF.to_vec())?;

    let input_data = to_vec(url).unwrap();
    let input_data = bytemuck::cast_slice(&input_data).to_vec();
    let input_id = client.upload_input(input_data)?;

    let assumptions: Vec<String> = vec![];
    let session = client.create_session(image_id, input_id, assumptions)?;

    loop {
        let res = session.status(&client)?;
        if res.status == "RUNNING" {
            eprintln!(
                "Current status: {} - state: {} - continue polling...",
                res.status,
                res.state.unwrap_or_default()
            );
            std::thread::sleep(Duration::from_secs(15));
            continue;
        }
        if res.status == "SUCCEEDED" {
            let receipt_url = res
                .receipt_url
                .expect("API error, missing receipt on completed session");
            let receipt_buf = client.download(&receipt_url)?;
            let receipt: Receipt = bincode::deserialize(&receipt_buf)?;
            receipt.verify(METHOD_ID).expect("Receipt verification failed");
            let verified_response: VerifiedResponse = receipt.journal.decode().unwrap();
            return Ok(verified_response);
        } else {
            panic!(
                "Workflow exited: {} - | err: {}",
                res.status,
                res.error_msg.unwrap_or_default()
            );
        }
    }
}

fn main() {
    let url = "https://api.someapi.com/somemethod?someparam=somevalue";
    match run_bonsai(url) {
        Ok(verified_response) => {
            println!("Verified Response: {:?}", verified_response);
            let is_valid = verify_response(&verified_response.request_hash, url);
            println!("Response validity: {}", is_valid);
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}