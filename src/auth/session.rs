//! Cookie-based session, secured by a HMAC signature.
use super::crypto;
use crate::{config, errors::ServerError, models::User, preferences};
use axum::headers::{HeaderMap, HeaderValue};
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Days, Utc};
use chrono_tz::Tz;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// `Session` is signed and serialized into the `Cookie` header when a
/// [HeaderMap] is passed into the [Session::update_headers()] method. Thus,
/// it's easy to extend this framework to store more information in the secure
/// session cookie by adding fields to this struct. However, keep in mind that
/// since this struct is serialized into a HTTP header, it cannot get too large!
///
/// # Serialization & Deserialization Note
///
/// This struct does derive [Serialize] and [Deserialize]. Internally, these
/// are used to serialize the struct into JSON. Then, the
/// [Session::from_headers()] and [Session::update_headers()] methods perform
/// some additonal ad-hoc serialization and deserialization to grep the session
/// string out of the Cookie string (where it is prefixed by `session=`), and
/// also to convert to/from base64 encoding.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub user: User,
    pub preferences: preferences::UserPreference,
    pub created_at: DateTime<Utc>,
}
impl Session {
    /// Parse the session from request headers, validating the cookie
    /// signature along the way. Returns the [None] variant if the session
    /// header is missing or invalid.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let cookie = headers.get("Cookie")?;
        let cookie = cookie.to_str().unwrap_or("");
        let re = Regex::new(r"session=(.*)").unwrap();
        let captures = re.captures(cookie)?;
        let token = &captures[1];
        let deserialize_result = Self::deserialize(token);

        if let Ok(session) = deserialize_result {
            Some(session)
        } else {
            None
        }
    }
    /// `err_msg` should identify which handler the error is coming from. Simply
    /// the name of the handler function is typically the best thing to put
    /// here.
    pub fn from_headers_err(
        headers: &HeaderMap,
        err_msg: &'static str,
    ) -> Result<Self, ServerError> {
        Self::from_headers(headers)
            .ok_or_else(|| ServerError::forbidden(err_msg))
    }
    /// Serialize the session into the provided [HeaderMap].
    pub fn update_headers(&self, mut headers: HeaderMap) -> HeaderMap {
        let session_string = self.serialize();
        let expiry_date = self
            .created_at
            .checked_add_days(Days::new(
                config::SESSION_EXPIRY_TIME_DAYS
                    .try_into()
                    .expect("7 can be a u64 too"),
            ))
            .expect("heat death of the universe has not happened yet")
            .with_timezone(&Tz::GMT)
            .format("%a, %d %b %Y %H:%M:%S %Z");

        let header_value = format!(
            "session={session_string}; Path=/; HttpOnly; Expires={expiry_date}"
        );
        headers.insert(
            "Set-Cookie",
            HeaderValue::from_str(&header_value).expect(
                "stringified session can be turned into a header value",
            ),
        );

        headers
    }
    fn serialize(&self) -> String {
        let json_bytes = serde_json::to_string(&self)
            .expect("session can be JSON serialized");
        let b64 = general_purpose::STANDARD_NO_PAD.encode(json_bytes);
        let raw_digest = crypto::get_digest(&b64.clone().into_bytes());
        let digest = general_purpose::STANDARD_NO_PAD.encode(raw_digest);
        let session = format!("{}:{}", b64, digest);

        session
    }
    fn deserialize(cookie: &str) -> Result<Self, &'static str> {
        let parts: Vec<&str> = cookie.split(':').collect();
        if parts.len() != 2 {
            Err("Invalid session")
        } else {
            let b64_json: Vec<u8> = parts[0].into();
            let digest: Vec<u8> =
                match general_purpose::STANDARD_NO_PAD.decode(parts[1]) {
                    Ok(v) => v,
                    Err(_) => {
                        return Err("Cannot base64 decode the digest");
                    }
                };

            if crypto::is_valid(&b64_json, &digest) {
                let json_string =
                    match general_purpose::STANDARD_NO_PAD.decode(b64_json) {
                        Ok(v) => v,
                        Err(_) => {
                            return Err("Cannot base64 decode sesion string");
                        }
                    };

                match serde_json::from_slice(&json_string) {
                    Ok(v) => Ok(v),
                    Err(_) => Err("Cannot deserialize session JSON"),
                }
            } else {
                Err("Failed to validate session signature")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use std::env;

    fn get_session() -> Session {
        Session {
            user: User {
                id: 1,
                username: "Jack".to_string(),
                email: "jack@jack.com".to_string(),
                stripe_customer_id: "".to_string(),
                stripe_subscription_type:
                    crate::stripe::SubscriptionTypes::Free,
                created_at: DateTime::<Utc>::from_timestamp(0, 0)
                    .expect("that is a valid timestamp"),
            },
            preferences: preferences::UserPreference {
                timezone: chrono_tz::Tz::US__Samoa,
                caloric_intake_goal: None,
            },
            created_at: DateTime::<Utc>::from_timestamp(0, 0)
                .expect("that is a valid timestamp"),
        }
    }

    const SERIALIZED_SESSION: &str = "eyJ1c2VyIjp7ImlkIjoxLCJ1c2VybmFtZSI6IkphY2siLCJlbWFpbCI6ImphY2tAamFjay5jb20iLCJjcmVhdGVkX2F0IjoiMTk3MC0wMS0wMVQwMDowMDowMFoiLCJzdHJpcGVfY3VzdG9tZXJfaWQiOiIiLCJzdHJpcGVfc3Vic2NyaXB0aW9uX3R5cGUiOiJGcmVlIn0sInByZWZlcmVuY2VzIjp7InRpbWV6b25lIjoiVVMvU2Ftb2EiLCJjYWxvcmljX2ludGFrZV9nb2FsIjpudWxsfSwiY3JlYXRlZF9hdCI6IjE5NzAtMDEtMDFUMDA6MDA6MDBaIn0:k85WWa60oKXRGXUlsreRMwLVz7qU0xOtor7025LMI9o";

    #[test]
    fn test_serialize_session() {
        env::set_var("SESSION_SECRET", "foo");

        let result = &get_session().serialize();
        // little snapshot test
        assert_eq!(result, SERIALIZED_SESSION);
    }

    #[test]
    fn test_deserialize_session() {
        env::set_var("SESSION_SECRET", "foo");

        let result = Session::deserialize(&String::from(SERIALIZED_SESSION))
            .expect("result");
        // little snapshot test
        assert_eq!(result.user.id, get_session().user.id);
    }
}
