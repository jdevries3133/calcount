use super::authenticate::authenticate;
use crate::{auth::InitAnonNextRoute, config, htmx, prelude::*};
use axum::http::{HeaderValue, StatusCode};

pub struct LoginForm;
impl Component for LoginForm {
    fn render(&self) -> String {
        let login_route = Route::Login;
        let password_reset = Route::PasswordReset;
        let init_anon = Route::InitAnon(InitAnonNextRoute::DefaultNextRoute);
        format!(
            r##"
            <div id="form-container">
                <form
                    id="login-form"
                    class="flex flex-col gap-2 max-w-md"
                    hx-post="{login_route}"
                    hx-target="#form-container"
                >
                    <h1 class="text-xl">Login to Bean Count</h1>
                    <label autocomplete="username" for="identifier">
                        Username or Email
                    </label>
                    <input
                        type="text"
                        id="identifier"
                        name="identifier"
                        autocomplete="username"
                    />
                    <label for="passwored">Password</label>
                    <input
                        autocomplete="current-password"
                        type="password"
                        id="password"
                        name="password"
                        />
                </form>
                <div class="flex gap-2 mt-3">
                    <button
                        type="submit"
                        form="login-form"
                        class="
                        bg-emerald-200
                        hover:bg-emerald-300
                        dark:bg-emerald-700
                        dark:hover:bg-emerald-600
                        dark:text-white
                        hover:shadow-none
                        p-1
                        rounded
                        shadow
                        transition
                        w-36
                        h-10
                    ">
                            Log In
                    </button>
                    <a href="{password_reset}">
                        <button class="
                            bg-yellow-200
                            hover:bg-yellow-300
                            dark:bg-yellow-700
                            dark:hover:bg-yellow-600
                            dark:text-white
                            hover:shadow-none
                            p-1
                            rounded
                            shadow
                            transition
                            w-36
                            h-10
                        ">
                            Reset Password
                        </button>
                    </a>
                    <form method="POST" action="{init_anon}">
                        <input type="hidden" value="" name="timezone" id="timezone" />
                        <script>
                            (() => {{
                                for (const el of document.querySelectorAll("[name='timezone'")) {{
                                    el.value = Intl.DateTimeFormat().resolvedOptions().timeZone;
                                }}
                            }})();
                        </script>
                        <button class="
                            border-emerald-200
                            border-2
                            hover:bg-emerald-200
                            dark:border-emerald-700
                            dark:hover:bg-emerald-700
                            dark:text-white
                            hover:shadow-none
                            p-1
                            rounded
                            shadow
                            transition
                            w-36
                            h-10
                        "
                        >Create Account</button>
                    </form>
                </div>
            </form>
            "##
        )
    }
}

pub async fn get_login_form(
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers);
    let form = if headers.contains_key("Hx-Request")
        && !headers.contains_key("Hx-Boosted")
    {
        LoginForm {}.render()
    } else {
        Page {
            title: "Login",
            children: &PageContainer {
                children: &LoginForm {},
            },
        }
        .render()
    };
    Ok(match session {
        Some(session) => {
            if utc_now()
                .signed_duration_since(session.created_at)
                .num_days()
                < config::SESSION_EXPIRY_TIME_DAYS
            {
                // The user is already authenticated, let's redirect them to the
                // user homepage.
                let mut headers = HeaderMap::new();
                headers.insert(
                    "Location",
                    HeaderValue::from_str(&Route::UserHome.as_string())?,
                );
                headers.insert(
                    "Hx-Redirect",
                    HeaderValue::from_str(&Route::UserHome.as_string())?,
                );

                (StatusCode::SEE_OTHER, headers).into_response()
            } else {
                form.into_response()
            }
        }
        None => form.into_response(),
    })
}

pub async fn logout() -> Result<impl IntoResponse, ServerError> {
    let login = Route::Login;
    let mut headers = HeaderMap::new();
    headers.insert(
        "Set-Cookie",
        HeaderValue::from_str("session=null; Path=/; HttpOnly")?,
    );
    headers.insert("Location", HeaderValue::from_str(&login.as_string())?);

    Ok((StatusCode::FOUND, headers))
}

#[derive(Debug, Deserialize)]
pub struct LoginFormPayload {
    /// Username or email
    identifier: String,
    password: String,
}

pub async fn handle_login(
    State(AppState { db }): State<AppState>,
    Form(form): Form<LoginFormPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let session = authenticate(&db, &form.identifier, &form.password).await;
    let headers = HeaderMap::new();
    if let Ok(session) = session {
        let homepage = Route::UserHome.as_string();
        let headers = session.update_headers(headers);
        let headers = htmx::redirect(headers, &homepage);
        Ok((headers, "OK".to_string()))
    } else {
        let login_route = Route::Login;
        Ok((
            headers,
            format!(
                r#"<p hx-trigger="load delay:1s" hx-get="{login_route}">Invalid login credentials.</p>"#
            ),
        ))
    }
}
