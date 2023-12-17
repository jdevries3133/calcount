//! Glue which integrates [crate::pw], [crate::db_ops], and [crate::session].
//! Auth will authenticate users by fetching user info from the database and
//! authenticating a user with the provided credentials.

use super::{
    db_ops, db_ops::DbModel, models, preferences::get_user_preference, pw,
    session,
};
use anyhow::{bail, Result};
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
) -> Result<session::Session> {
    let user = models::User::get(
        db,
        &db_ops::GetUserQuery {
            identifier: username_or_email,
        },
    )
    .await?;
    let preferences = get_user_preference(db, &user).await?.unwrap_or_default();
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
        let now: i64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            .try_into()
            .expect("today can fit into i64");
        Ok(session::Session {
            user,
            preferences,
            created_at: now,
        })
    } else {
        bail!("wrong password")
    }
}
