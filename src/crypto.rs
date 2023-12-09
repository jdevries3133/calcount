//! Password hashing! Note that this module doesn't have any of the
//! cryptographic code for [crate::session], which I'm now realizing doesn't
//! really make sense, does it?

use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;

type HmacSha256 = Hmac<Sha256>;

fn get_session_secret() -> Vec<u8> {
    env::var("SESSION_SECRET")
        .expect("session secret to be defined in the environment")
        .into()
}

pub fn get_digest(val: &[u8]) -> Vec<u8> {
    let secret = get_session_secret();
    let mut mac =
        HmacSha256::new_from_slice(&secret).expect("can init with secret key");
    mac.update(val);

    mac.finalize().into_bytes().to_vec()
}

pub fn is_valid(val: &[u8], digest: &[u8]) -> bool {
    let secret = get_session_secret();
    let mut mac =
        HmacSha256::new_from_slice(&secret).expect("can init with secret key");
    mac.update(val);

    mac.verify_slice(digest).is_ok()
}
