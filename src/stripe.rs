use crate::prelude::*;
use anyhow::Result as Aresult;
use base64::{engine::general_purpose, Engine as _};
use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use sha2::Sha256;
use std::{collections::HashMap, env};

#[cfg(feature = "use_stripe_test_instance")]
const BASIC_PLAN_STRIPE_ID: &str = "price_1OOr4nAaiRLwV5fgUhgO8ZRT";
#[cfg(feature = "use_stripe_test_instance")]
#[cfg(feature = "use_stripe_test_instance")]
const BILLING_PORTAL_CONFIGURATION_ID: &str = "bpc_1OOqe5AaiRLwV5fgrDmCz5xE";

#[cfg(not(feature = "use_stripe_test_instance"))]
const BASIC_PLAN_STRIPE_ID: &str = "price_1OOr4nAaiRLwV5fgUhgO8ZRT";
#[cfg(feature = "use_stripe_test_instance")]
#[cfg(not(feature = "use_stripe_test_instance"))]
const BILLING_PORTAL_CONFIGURATION_ID: &str = "bpc_1OOqe5AaiRLwV5fgrDmCz5xE";

#[derive(Debug, Serialize, Deserialize)]
pub enum SubscriptionTypes {
    Initializing,
    Basic,
    Free,
    Unsubscribed,
    FreeTrial,
}

impl SubscriptionTypes {
    pub fn as_int(&self) -> i32 {
        match self {
            Self::Initializing => 1,
            Self::Basic => 2,
            Self::Free => 3,
            Self::Unsubscribed => 4,
            Self::FreeTrial => 5,
        }
    }
    pub fn from_int(int: i32) -> Self {
        match int {
            1 => Self::Initializing,
            2 => Self::Basic,
            3 => Self::Free,
            4 => Self::Unsubscribed,
            5 => Self::FreeTrial,
            n => panic!("{n} is an invalid subscription type"),
        }
    }
}

/// This is my own simple and sane data-model for a stripe webhook event.
struct StripeUpdate {
    stripe_customer_id: String,
    subscription_type: SubscriptionTypes,
}

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

#[derive(Serialize)]
struct BillingPortalRequest {
    customer: String,
    return_url: String,
    configuration: String,
}

#[cfg(feature = "stripe")]
#[derive(Deserialize)]
struct BillingPortalResponse {
    url: String,
}

#[cfg(feature = "stripe")]
/// Returns the URL for the billing session, to which the customer can be
/// redirected.
pub async fn get_billing_portal_url(
    stripe_customer_id: &str,
) -> Aresult<String> {
    let url = "https://api.stripe.com/v1/billing_portal/sessions";
    let secret_key = get_b64_encoded_token_from_env()?;
    let return_url = "https://beancount.bot/home";
    let request_payload = BillingPortalRequest {
        customer: stripe_customer_id.to_string(),
        return_url: return_url.to_string(),
        configuration: BILLING_PORTAL_CONFIGURATION_ID.to_string(),
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
pub async fn get_billing_portal_url(_customer_id: &str) -> Aresult<String> {
    Ok(Route::UserHome.as_string())
}

fn get_b64_encoded_token_from_env() -> Aresult<String> {
    let secret_key = env::var("STRIPE_API_KEY")?;
    Ok(general_purpose::STANDARD_NO_PAD.encode(secret_key))
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

    if !r#type.starts_with("customer.subscription") {
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
                SubscriptionStatus::Trialing => SubscriptionTypes::FreeTrial,
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

async fn persist_update(db: &PgPool, update: &StripeUpdate) -> Aresult<()> {
    query!(
        "update users set subscription_type_id = $1
        where stripe_customer_id = $2",
        update.subscription_type.as_int(),
        update.stripe_customer_id
    )
    .execute(db)
    .await?;
    Ok(())
}

pub async fn handle_stripe_webhook(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, ServerError> {
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
    .execute(&db)
    .await?;
    if let Some(update) = parsed_update {
        persist_update(&db, &update).await?;
    };
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
