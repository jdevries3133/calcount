//! A GPT-powered calorie counter.

use anyhow::Result;
use dotenvy::dotenv;
use std::net::SocketAddr;

mod auth;
mod chrono_utils;
mod client_events;
mod components;
mod config;
mod controllers;
mod count_chat;
mod db_ops;
mod errors;
mod htmx;
mod legal;
mod metrics;
mod middleware;
mod models;
mod preferences;
mod prelude;
mod routes;
mod smtp;
mod stripe;
mod timeutils;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let db = db_ops::create_pg_pool().await?;
    sqlx::migrate!().run(&db).await?;
    let state = models::AppState { db };

    let app = routes::get_routes(state.clone()).with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
