//! Our own UI around Stripe integration. Includes:
//!
//! - success page
//! - cancel page
//! - trial expiry pages
//! - customer portal link helpers

use super::env::get_b64_encoded_token_from_env;
use crate::{config, prelude::*};
use axum::response::Redirect;
use reqwest::Client;
use serde::Serialize;

#[derive(Serialize)]
struct BillingPortalRequest<'a> {
    customer: &'a str,
    return_url: &'a str,
}

#[derive(Deserialize)]
struct BillingPortalResponse {
    url: String,
}

pub async fn redirect_to_billing_portal(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session =
        Session::from_headers_err(&headers, "redirect to billing portal")?;
    let user = session.get_user(&db).await?;
    let url = "https://api.stripe.com/v1/billing_portal/sessions";
    let secret_key = get_b64_encoded_token_from_env()?;
    let return_url =
        format!("{}{}", config::BASE_URL, Route::UserHome.as_string());
    let request_payload = BillingPortalRequest {
        customer: &user.stripe_customer_id,
        return_url: &return_url,
    };
    let client = Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Basic {secret_key}"))
        .form(&request_payload)
        .send()
        .await?;
    if response.status().is_success() {
        let url = response.json::<BillingPortalResponse>().await?.url;
        Ok(Redirect::to(&url))
    } else {
        Err(Error::msg("request to create registration session failed (redirect to billing portal)").into())
    }
}
