//! Database operations; squirrel code lives here.

use super::{auth::HashedPw, models, models::IdCreatedAt, stripe};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    query_as,
};
use std::collections::HashSet;

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
        struct Qres {
            id: i32,
            username: String,
            email: String,
            stripe_customer_id: String,
            subscription_type_id: i32,
            created_at: DateTime<Utc>,
        }
        Ok(query_as!(
            Qres,
            "select
                id,
                username,
                email,
                stripe_customer_id,
                subscription_type_id,
                created_at
            from users
            where username = $1 or email = $1",
            query.identifier
        )
        .try_map(|row| {
            Ok(Self {
                stripe_subscription_type: stripe::SubscriptionTypes::from_int(
                    row.subscription_type_id,
                ),
                id: row.id,
                username: row.username,
                email: row.email,
                stripe_customer_id: row.stripe_customer_id,
                created_at: row.created_at,
            })
        })
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
    pw: &HashedPw,
    stripe_customer_id: String,
    subscription_type: stripe::SubscriptionTypes,
) -> Result<models::User> {
    let query_return = query_as!(
        IdCreatedAt,
        "insert into users
        (
            username,
            email,
            salt,
            digest,
            stripe_customer_id,
            subscription_type_id
        )
         values ($1, $2, $3, $4, $5, $6)
        returning id, created_at",
        username,
        email,
        pw.salt,
        pw.digest,
        stripe_customer_id,
        subscription_type.as_int()
    )
    .fetch_one(db)
    .await?;

    Ok(models::User {
        id: query_return.id,
        created_at: query_return.created_at,
        username,
        email,
        stripe_customer_id,
        stripe_subscription_type: subscription_type,
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
