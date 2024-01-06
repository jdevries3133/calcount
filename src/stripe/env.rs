use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use std::env;

pub fn get_b64_encoded_token_from_env() -> Result<String> {
    let secret_key = env::var("STRIPE_API_KEY")?;
    Ok(general_purpose::STANDARD_NO_PAD.encode(secret_key))
}
