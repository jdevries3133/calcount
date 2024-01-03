//! Axum middlewares, modeled as async functions.

use super::{config, htmx, routes::Route, session};
use axum::{
    http::{HeaderMap, HeaderValue, Request},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use chrono::prelude::*;

/// This will ensure that outgoing requests receive a content-type if the
/// request handler did not specify one. 99% of request handlers in this
/// application are returning a content-type of HTML.
///
/// Note that Axum by default applies a content-type of `text/plain` to outgoing
/// requests. We are going to step on the toes of any _real_ `text/plain`
/// responses on their way out the door, and change this to `text/html`.
///
/// This middleware also ensures that we have `Cache-Control: no-cache` on
/// any responses where cache-control is not specify. This is important because
/// all of my websites run behind Cloudflare, so we need to ensure that
/// we're being explicit about caching.
pub async fn html_headers<B>(request: Request<B>, next: Next<B>) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    // Set content-type to text/html unless otherwise specified
    if let Some(content_type) = headers.get("content-type") {
        if content_type.to_str().expect("header is ascii")
            == "text/plain; charset=utf-8"
        {
            headers.remove("content-type");
            headers.insert(
                "content-type",
                HeaderValue::from_str("text/html").expect("text/html is ascii"),
            );
        }
    }
    // Set Cache-Control: no-cache unless otherwise specified. Most endpoints
    // return HTML interpolated with user data which is liable to change all
    // the time, so we don't want these responses to be cached. At least one
    // route, though, does have some specific cache-control. The route to serve
    // static JS can be cached forever.
    if !headers.contains_key("cache-control") {
        headers.insert(
            "cache-control",
            HeaderValue::from_str("no-cache").expect("no-cache is ascii"),
        );
    };

    response
}

/// This will validate the session from the request headers and redirect any
/// unauthenticated users to the login route, allowing the creation of a
/// router with protected routes for users only. Unfortunately, this work
/// is not passed along to request handlers because I don't know how, so the
/// session parsing work will be repeated, but these are JWT-style tokens, so
/// validating the session at least does not require a database round trip. This
/// middleware also logs the method, path, and username for authenticated
/// requests.
pub async fn auth<B>(request: Request<B>, next: Next<B>) -> Response {
    let headers = request.headers();
    let session = session::Session::from_headers(headers);

    // We want to perform a htmx redirect with the Hx-Redirect header in
    // addition to a regular browser redirect if the user is not authenticated.
    // Otherwise, a hx-get request might visit an authenticated route and then
    // receive the login form as a response, since htmx just follows the
    // browser redirect to get the final content. It's a bit weird to click
    // a button and have the login form pop up inline inside pages!!
    let response_headers = || {
        let h = HeaderMap::new();
        htmx::redirect(h, &Route::Login.to_string())
    };

    if let Some(session) = session {
        if let Some(created_time) =
            NaiveDateTime::from_timestamp_opt(session.created_at, 0)
        {
            let token_age_days = Local::now()
                .naive_local()
                .signed_duration_since(created_time)
                .num_days();
            if token_age_days < config::SESSION_EXPIRY_TIME_DAYS {
                let path = request.uri().path();
                let method = request.method().as_str();
                let username = session.user.username;
                println!("{method} {path} from {username}");
                next.run(request).await
            } else {
                let username = session.user.username;
                println!("{username} has an expired token (created at {created_time}, is {token_age_days} days old)");
                (response_headers(), Redirect::to(&Route::Login.to_string()))
                    .into_response()
            }
        } else {
            (response_headers(), Redirect::to(&Route::Login.to_string()))
                .into_response()
        }
    } else {
        (response_headers(), Redirect::to(&Route::Login.to_string()))
            .into_response()
    }
}

pub async fn log<B>(request: Request<B>, next: Next<B>) -> Response {
    let uri = request.uri().path();
    let method = request.method().as_str();
    println!("{method} {uri} (anonymous)");
    next.run(request).await
}
