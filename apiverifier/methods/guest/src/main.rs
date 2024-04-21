use apiverifier::VerifiedResponse;
use risc0_zkvm::guest::env;

risc0_zkvm::guest::entry!(main);

pub fn main() {
    let url = env::read();
    match fetch_data(&url) {
        Ok(verified_response) => {
            env::commit(&verified_response);
        },
        Err(e) => {
            panic!("Error fetching data: {}", e);
        }
    }
}