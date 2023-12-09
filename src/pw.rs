//! Methods for handling passwords. Passwords are combined with random salt and
//! hashed using [sha2::Sha256]. Only the salt and resultant digest is then
//! persisted to the `users` table in the database. Salt comes from UUIDv4,
//! which
//! [is not a secure source of salt](https://stackoverflow.com/a/3596660/13262536),
//! so please use long and secure passwords, and PRs to fix this are very
//! welcome!

use anyhow::{bail, Result};
use base64::{engine::general_purpose, Engine};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Default)]
pub struct HashedPw {
    pub salt: String,
    pub digest: String,
}

fn hash(pw: &str, salt: &str) -> HashedPw {
    let mut pw_digest = salt.to_string();
    pw_digest.push_str(pw);

    let mut hash = Sha256::new();
    hash.update(&pw_digest);

    let digest = hash.finalize().to_vec();
    let b64_digest = general_purpose::STANDARD.encode(digest);

    if b64_digest.len() > 255 {
        panic!("password is too long");
    }

    HashedPw {
        salt: salt.to_string(),
        digest: b64_digest,
    }
}

/// Hash a new password; random salt is generated.
pub fn hash_new(pw: &str) -> HashedPw {
    let salt = Uuid::new_v4();
    let salt_str = salt.clone().to_string();

    hash(pw, &salt_str)
}

/// Check if a user's password `pw` matches a HashedPw from the database
/// `truth`
///
/// # Bugs
///
/// This does not use a cryptographically secure string comparison, and may
/// therefore be vulnerable to timing attack.
pub fn check(pw: &str, truth: &HashedPw) -> Result<()> {
    let user_input_digest = hash(pw, &truth.salt).digest;

    if user_input_digest == truth.digest {
        Ok(())
    } else {
        bail!("passwords do not match")
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_can_hash_new_pw() {
        hash_new("heyoooo");
    }

    #[test]
    fn test_can_check_old_pw() {
        let hash = hash_new("heyoooo");
        check("heyoooo", &hash).unwrap();
    }

    #[test]
    fn test_utf8_support() {
        let hash = hash_new("heyoooo 🥹");
        check("heyoooo 🥹", &hash).unwrap();
    }
}
