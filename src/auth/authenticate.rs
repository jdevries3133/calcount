//! Glue which integrates [crate::pw], [crate::db_ops], and [crate::session].
//! Auth will authenticate users by fetching user info from the database and
//! authenticating a user with the provided credentials.

use super::{pw, Session};
use crate::{db_ops, db_ops::DbModel, models};
use anyhow::{bail, Result};
use chrono::Utc;
use sqlx::{postgres::PgPool, query_as};

/// We are a bit losey goosey on the identifier for a better user experience.
/// I'm fairly convinced this is not a security issue. If we consider a
/// malicious user who creates an account where their username is someone else's
/// email, or their email is someone else's username, then they could certainly
/// get into a position where the `user` who is fetched by our database query
/// here is the target victim's user, and not the attacker's user. However,
/// the `truth` (the known password digest) is ultimately associated with the
/// target victim's password. So, an attaker would need to know the target
/// victim's password to be able to authenticate as that user.
pub async fn authenticate(
    db: &PgPool,
    username_or_email: &str,
    password: &str,
) -> Result<Session> {
    let user = models::User::get(
        db,
        &db_ops::GetUserQuery {
            identifier: db_ops::UserIdentifer::Identifier(username_or_email),
        },
    )
    .await?;
    // This is kept out of the `User` model because I don't want to leak
    // password digests in autnentication tokens. The entire User object is
    // serialized into the user's session token, which is signed but not
    // encrypted.
    let truth = query_as!(
        pw::HashedPw,
        "SELECT salt, digest FROM users WHERE id = $1",
        user.id
    )
    .fetch_one(db)
    .await?;

    if pw::check(password, &truth).is_ok() {
        Ok(Session {
            user_id: user.id,
            username: user.username.clone(),
            created_at: Utc::now(),
        })
    } else {
        bail!("wrong password")
    }
}
