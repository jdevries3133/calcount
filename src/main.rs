//! A notion clone!

use anyhow::Result;
use axum::{middleware::from_fn, Router};
use dotenvy::dotenv;
use std::net::SocketAddr;

mod auth;
mod components;
mod config;
mod controllers;
mod crypto;
mod db_ops;
mod errors;
mod htmx;
mod middleware;
mod models;
mod pw;
mod routes;
mod session;

/// The Notion Clone entrypoint. Note that I envision this binary some day
/// becoming a CLI to support the prod backfill operations from our propval
/// system. See also [DESIGN.md on
/// GitHub](https://github.com/jdevries3133/nc/blob/main/DESIGN.md#page-props-can-backfill-the-database).
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
