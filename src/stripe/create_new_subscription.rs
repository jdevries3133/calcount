//! Interface to the stripe API

use super::{
    db_ops::persist_update_op,
    env::get_b64_encoded_token_from_env,
    models::{StripeUpdate, SubscriptionTypes},
};
use crate::{config, prelude::*};
use anyhow::Result as Aresult;
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;
use std::{collections::HashMap, env};

#[cfg(feature = "use_stripe_test_instance")]
const BASIC_PLAN_STRIPE_ID: &str = "price_1OTyEXBhmccJFhTPvs01VoJf";

#[cfg(not(feature = "use_stripe_test_instance"))]
const BASIC_PLAN_STRIPE_ID: &str = "price_1OOr4nAaiRLwV5fgUhgO8ZRT";

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
        .header("Authorization", format!("Basic {}:", secret_key));
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
        price: BASIC_PLAN_STRIPE_ID.to_string(),
        quantity: 1,
        mode: "subscription".to_string(),
    };
    let client = Client::new();
    let response = client
        .post(url)
        .header("Authorization", format!("Basic {}", secret_key))
        .form(&request_payload)
        .send()
        .await?;
    if response.status().is_success() {
        Ok(response.json::<BillingPortalResponse>().await?.url)
    } else {
        Err(Error::msg("request to create registration session failed"))
    }
}

#[cfg(not(feature = "stripe"))]
pub async fn get_basic_plan_checkout_session(
    _customer_id: &str,
) -> Aresult<String> {
    Ok(Route::UserHome.as_string())
}

#[derive(Deserialize)]
struct StripeSubscriptionUpdate {
    data: StripeWrapper,
}

#[derive(Deserialize)]
struct StripeWrapper {
    object: InnerSubscriptionUpdated,
}

#[derive(Deserialize)]
struct InnerSubscriptionUpdated {
    customer: String,
    status: SubscriptionStatus,
    items: StripeWrapperAgain,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum SubscriptionStatus {
    Incomplete,
    IncompleteExpired,
    Trialing,
    Active,
    PastDue,
    Canceled,
    Unpaid,
}

#[derive(Deserialize)]
struct StripeWrapperAgain {
    data: Vec<SubscriptionUpdateItems>,
}

#[derive(Deserialize)]
struct SubscriptionUpdateItems {
    price: SubscriptionPrice,
}

#[derive(Deserialize)]
struct SubscriptionPrice {
    id: String,
}

fn parse_update(stripe_garbage: &str) -> Option<StripeUpdate> {
    let data: Value = serde_json::from_str(stripe_garbage).ok()?;
    let r#type = data.get("type")?.as_str()?;

    #[allow(clippy::if_same_then_else)]
    if !r#type.starts_with("customer.subscription") {
        None
    } else if r#type == "customer.subscription.created" {
        // Subscription created events always have a subscription status of
        // "incomplete," and then stripe _very very quickly_ follows-up with
        // a subscription updated event where they say that the subscription
        // is active. So fast, in fact, that we'll get race conditions if
        // we handle the created event after the updated event -__-
        //
        // It seems like we can just ignore the subscription created event.
        // IDK why the hell stripe would do this.
        //
        // https://news.ycombinator.com/item?id=19608955
        None
    } else {
        let subscription: StripeSubscriptionUpdate =
            serde_json::from_str(stripe_garbage).ok()?;
        let is_relevant = subscription
            .data
            .object
            .items
            .data
            .iter()
            .any(|i| i.price.id == BASIC_PLAN_STRIPE_ID);
        if is_relevant {
            let sub_ty = match subscription.data.object.status {
                SubscriptionStatus::Active => SubscriptionTypes::Basic,
                SubscriptionStatus::Unpaid
                | SubscriptionStatus::PastDue
                | SubscriptionStatus::Canceled
                | SubscriptionStatus::Incomplete
                | SubscriptionStatus::IncompleteExpired => {
                    SubscriptionTypes::Unsubscribed
                }
                SubscriptionStatus::Trialing => {
                    SubscriptionTypes::FreeTrial(config::FREE_TRIAL_DURATION)
                }
            };
            Some(StripeUpdate {
                stripe_customer_id: subscription.data.object.customer,
                subscription_type: sub_ty,
            })
        } else {
            None
        }
    }
}

pub async fn handle_stripe_webhook(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, ServerError> {
    let mut tx = db.begin().await?;
    query!("lock audit_stripe_webhooks")
        .execute(&mut tx)
        .await?;
    let signature = headers
        .get("Stripe-Signature")
        .ok_or(Error::msg("signature is missing"))?
        .to_str()?;
    authenticate_stripe(signature, &body)?;
    let parsed_update = parse_update(&body);
    query!(
        "insert into audit_stripe_webhooks (payload, includes_usable_update)
        values ($1, $2)
        ",
        body,
        parsed_update.is_some()
    )
    .execute(&mut tx)
    .await?;
    if let Some(update) = parsed_update {
        println!("persisting relevant stripe update: {update:?}");
        persist_update_op(&mut tx, &update).await?;
    };
    tx.commit().await?;
    Ok("")
}

fn authenticate_stripe(
    signature_header: &str,
    message_body: &str,
) -> Aresult<()> {
    let parts = signature_header.split(',');
    let mut entries = HashMap::new();
    for part in parts {
        let mut iter = part.split('=');
        let key = iter.next().unwrap_or_default();
        let value = iter.next().unwrap_or_default();
        entries.insert(key, value);
    }
    let timestamp =
        *entries.get("t").ok_or(Error::msg("timestamp is missing"))?;
    let timestamp_dt = timestamp.parse::<i64>()?;
    let now = Utc::now().timestamp();
    let time_diff = if (timestamp_dt - now).is_negative() {
        now - timestamp_dt
    } else {
        timestamp_dt - now
    };
    let is_too_old = time_diff > 60;
    let external_digest =
        *entries.get("v1").ok_or(Error::msg("digest is missing"))?;
    let external_digest = external_digest.as_bytes();
    let payload_str = format!("{}.{}", timestamp, message_body);
    let payload = payload_str.as_bytes();
    let signing_secret = env::var("STRIPE_WEBHOOK_SIGNING_SECRET")?;
    let mut mac = Hmac::<Sha256>::new_from_slice(signing_secret.as_bytes())?;
    mac.update(payload);
    let sig = hex::decode(external_digest)?;
    mac.verify_slice(sig.as_slice())?;
    if !is_too_old {
        Ok(())
    } else {
        Err(Error::msg("signature does not match"))
    }
}
