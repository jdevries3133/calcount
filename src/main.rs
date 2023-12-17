//! A GPT-powered calorie counter.

use anyhow::Result;
use axum::{middleware::from_fn, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

mod auth;
mod chrono_utils;
mod client_events;
mod components;
mod config;
mod controllers;
mod count_chat;
mod crypto;
mod db_ops;
mod errors;
mod htmx;
mod metrics;
mod middleware;
mod models;
mod preferences;
mod prelude;
mod pw;
mod routes;
mod session;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let db = db_ops::create_pg_pool().await?;
    sqlx::migrate!().run(&db).await?;
    let state = models::AppState { db };
    let routes = routes::get_protected_routes()
        .layer(from_fn(middleware::html_headers))
        .layer(from_fn(middleware::auth));

    let public_routes =
        routes::get_public_routes().layer(from_fn(middleware::html_headers));

    let app = Router::new()
        .nest("/", routes)
        .nest("/", public_routes)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
