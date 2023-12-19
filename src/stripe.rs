use crate::prelude::*;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use std::env;

#[derive(Deserialize)]
struct BillingPortalSession {
    id: String,
}

/// Returns the stripe customer ID
pub async fn create_customer(name: &str, email: &str) -> Result<String> {
    let url = "https://api.stripe.com/v1/customers";
    let secret_key =
        env::var("STRIPE_API_KEY").expect("stripe API key is available");
    let b64_token = general_purpose::STANDARD_NO_PAD.encode(secret_key);
    let params = [("name", name), ("email", email)];

    let client = Client::new();
    let builder = client
        .post(url)
        .header("Authorization", format!("Basic {}:", b64_token));
    let builder = builder.form(&params);
    let response: BillingPortalSession = builder.send().await?.json().await?;
    Ok(response.id)
}
