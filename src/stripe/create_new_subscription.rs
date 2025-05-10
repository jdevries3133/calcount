//! Interface to the stripe API

use super::env::get_b64_encoded_token_from_env;
#[cfg(feature = "stripe")]
use crate::config;
use crate::prelude::*;
use anyhow::Result as Aresult;
use reqwest::Client;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
struct BillingPortalSession {
    id: String,
}

/// Returns the stripe customer ID
pub async fn create_customer(name: &str, email: &str) -> Aresult<String> {
    let url = "https://api.stripe.com/v1/customers";
    let secret_key = get_b64_encoded_token_from_env()?;
    let params = [("name", name), ("email", email)];

    let client = Client::new();
    let builder = client
        .post(url)
        .header("Authorization", format!("Basic {secret_key}:"));
    let builder = builder.form(&params);
    let response: BillingPortalSession = builder.send().await?.json().await?;
    Ok(response.id)
}

#[derive(Debug, Serialize)]
struct BillingPortalRequest {
    customer: String,
    success_url: String,
    #[serde(rename = "line_items[0][price]")]
    price: String,
    #[serde(rename = "line_items[0][quantity]")]
    quantity: i32,
    mode: String,
}

#[derive(Debug, Deserialize)]
#[cfg(feature = "stripe")]
struct BillingPortalResponse {
    url: String,
}

#[cfg(feature = "stripe")]
/// Returns the URL for the billing session, to which the customer can be
/// redirected.
pub async fn get_basic_plan_checkout_session(
    stripe_customer_id: &str,
) -> Aresult<String> {
    let url = "https://api.stripe.com/v1/checkout/sessions";
    let secret_key = get_b64_encoded_token_from_env()?;
    let success_url = format!("{}{}", config::BASE_URL, Route::UserHome);
    let request_payload = BillingPortalRequest {
        customer: stripe_customer_id.to_string(),
        success_url: success_url.to_string(),
        price: config::BASIC_PLAN_STRIPE_ID.to_string(),
        quantity: 1,
        mode: "subscription".to_string(),
    };
    let client = Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Basic {secret_key}"))
        .form(&request_payload)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json::<BillingPortalResponse>().await?.url)
    } else {
        Err(Error::msg("request to create registration session failed (get basic plan checkout)"))
    }
}

#[cfg(not(feature = "stripe"))]
pub async fn get_basic_plan_checkout_session(
    _customer_id: &str,
) -> Aresult<String> {
    Ok(Route::UserHome.as_string())
}
