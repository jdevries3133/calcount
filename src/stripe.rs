use crate::prelude::*;
use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use reqwest::Client;
use serde::Serialize;
use std::env;

#[derive(Deserialize, Serialize)]
struct BillingPortalSession {
    id: String,
}

/// Returns the stripe customer ID
pub async fn create_customer(name: &str, email: &str) -> Result<String> {
    let url = "https://api.stripe.com/v1/customers";
    let secret_key = get_b64_encoded_token_from_env()?;
    let params = [("name", name), ("email", email)];

    let client = Client::new();
    let builder = client
        .post(url)
        .header("Authorization", format!("Basic {}:", secret_key));
    let builder = builder.form(&params);
    let response: BillingPortalSession = builder.send().await?.json().await?;
    Ok(response.id)
}

#[derive(Serialize)]
struct BillingPortalRequest {
    customer: String,
    return_url: String,
}

#[derive(Deserialize)]
struct BillingPortalResponse {
    url: String,
}

/// Returns the URL for the billing session, to which the customer can be
/// redirected.
pub async fn create_billing_portal_session(
    stripe_customer_id: &str,
) -> Result<String> {
    let url = "https://api.stripe.com/v1/billing_portal/sessions";

    let secret_key = get_b64_encoded_token_from_env()?;

    let return_url = "https://calcount.jackdevries.com/home";

    // Create the request payload
    let request_payload = BillingPortalRequest {
        customer: stripe_customer_id.to_string(),
        return_url: return_url.to_string(),
    };

    // Create a reqwest client with the Authorization header
    let client = Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Basic {}", secret_key))
        .form(&request_payload)
        .send()
        .await?;

    // Check if the request was successful (status code 2xx)
    if response.status().is_success() {
        Ok(response.json::<BillingPortalResponse>().await?.url)
    } else {
        Err(Error::msg("request to create reigstration session failed"))
    }
}

fn get_b64_encoded_token_from_env() -> Result<String> {
    let secret_key = env::var("STRIPE_API_KEY")?;
    Ok(general_purpose::STANDARD_NO_PAD.encode(secret_key))
}
