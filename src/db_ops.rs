//! Database operations; squirrel code lives here.

use super::{models, pw};
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    query_as,
};
use std::collections::HashSet;

/// Generic container for database IDs. For example, to be used with queries
/// returning (id).
struct Id {
    id: i32,
}

pub async fn create_pg_pool() -> Result<sqlx::Pool<sqlx::Postgres>> {
    let db_url = &std::env::var("DATABASE_URL")
        .expect("database url to be defined in the environment")[..];

    Ok(PgPoolOptions::new()
        // Postgres default max connections is 100, and we'll take 'em
        // https://www.postgresql.org/docs/current/runtime-config-connection.html
        .max_connections(80)
        .connect(db_url)
        .await?)
}

/// Note that in this starter repo, this trait only has one implementer;
/// `models::User`. This trait also doesn't really make much sense for the
/// user model, though it does make sense for other object types.
///
/// Notet that this trait as it is definitely has some issues.
///
/// - methods receive [sqlx::postgres::PgPool] as inputs, so we can't use
///   operations within transaction isolation because we can't pass transactions
///   to them.
/// - we're not passing any `user_id` or other authorization info into database
///   operations. I actually never got to the point of adding authorization into
///   my Notion clone, so this is something that would certainly needed to be
///   added.
#[async_trait]
pub trait DbModel<GetQuery, ListQuery>: Sync + Send {
    /// Get exactly one object from the database, matching the query. WIll
    /// return an error variant if the item does not exist.
    async fn get(db: &PgPool, query: &GetQuery) -> Result<Self>
    where
        Self: Sized;
    /// Get a set of objects from the database, matching the contents of the
    /// list query type.
    async fn list(db: &PgPool, query: &ListQuery) -> Result<Vec<Self>>
    where
        Self: Sized;
    /// Persist the object to the database
    async fn save(&self, db: &PgPool) -> Result<()>;
    /// Delete the record from the databse, which could of course cascade
    /// to related rows based on the rules in the database schema for this
    /// table.
    ///
    /// Delete will consume `self`.
    ///
    /// Most `.save` methods are implemented using update queries, under the
    /// assumption that the object already exists and we are just mutating it
    /// and then calling `.save` to persist the mutation. Deletion, then,
    /// would naturally invalidate these save queries.
    ///
    /// Additionally, a delete operation can trigger cascading deletion,
    /// so the existing record will often change structurally after deletion,
    /// because other rows around it will be deleted as well. The strategy
    /// for recovering from deletion will vary based on the object type,
    /// which is why the delete method consumes `self`.
    async fn delete(self, _db: &PgPool) -> Result<()>;
}

pub struct GetUserQuery<'a> {
    /// `identifier` can be a users username _or_ email
    pub identifier: &'a str,
}

#[async_trait]
impl DbModel<GetUserQuery<'_>, ()> for models::User {
    async fn get(db: &PgPool, query: &GetUserQuery) -> Result<Self> {
        Ok(query_as!(
            Self,
            "select id, username, email from users
            where username = $1 or email = $1",
            query.identifier
        )
        .fetch_one(db)
        .await?)
    }
    async fn list(_db: &PgPool, _query: &()) -> Result<Vec<Self>> {
        todo!()
    }
    async fn save(&self, _db: &PgPool) -> Result<()> {
        todo!()
    }
    async fn delete(self, _db: &PgPool) -> Result<()> {
        todo!();
    }
}

pub async fn create_user(
    db: &PgPool,
    username: String,
    email: String,
    pw: &pw::HashedPw,
) -> Result<models::User> {
    let id = query_as!(
        Id,
        "insert into users (username, email, salt, digest) values ($1, $2, $3, $4)
        returning id",
        username, email, pw.salt, pw.digest
    ).fetch_one(db).await?;

    Ok(models::User {
        id: id.id,
        username,
        email,
    })
}

pub async fn get_registraton_keys(db: &PgPool) -> Result<HashSet<String>> {
    struct Qres {
        key: String,
    }
    let mut keys = query_as!(Qres, "select key from registration_key")
        .fetch_all(db)
        .await?;
    Ok(keys.drain(..).fold(HashSet::new(), |mut acc, k| {
        acc.insert(k.key);
        acc
    }))
}
