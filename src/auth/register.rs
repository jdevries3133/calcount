use super::pw;
use crate::{
    components::Span,
    config,
    db_ops::{GetModel, GetUserQuery, SaveModel},
    html_sanitize::encode_quotes,
    htmx,
    models::{IdCreatedAt, User},
    preferences::save_user_preference,
    prelude::*,
    stripe,
};
use ammonia::clean;
use axum::http::StatusCode;
use futures::join;
use std::default::Default;

#[derive(Default)]
pub struct RegisterForm<'a> {
    username: Option<&'a str>,
    email: Option<&'a str>,
    password: Option<&'a str>,
    errors: Option<&'a dyn Component>,
}
impl Component for RegisterForm<'_> {
    fn render(&self) -> String {
        let register_route = Route::Register;
        let login_route = Route::Login;
        let username = encode_quotes(&clean(self.username.unwrap_or_default()));
        let email = encode_quotes(&clean(self.email.unwrap_or_default()));
        let password = encode_quotes(&clean(self.password.unwrap_or_default()));
        let error_msg = self.errors.map_or("".into(), |e| e.render());
        format!(
            r#"
            <form class="flex flex-col gap-2 max-w-md" hx-post="{register_route}">
                <h1 class="text-xl">Register for a Bean Count Account</h1>
                {error_msg}
                <label for="username">Username</label>
                <input value="{username}" autocomplete="username" type="text" id="username" name="username" />
                <label for="email">Email</label>
                <input value="{email}" type="email" id="email" name="email" />
                <label for="password">Password</label>
                <input
                    value="{password}"
                    autocomplete="current-password"
                    type="password"
                    id="password"
                    name="password"
                />
                <input type="hidden" value="" name="timezone" id="timezone" />
                <script>
                    (() => {{
                        const el = document.getElementById("timezone");
                        el.value = Intl.DateTimeFormat().resolvedOptions().timeZone;
                    }})();
                </script>
                <div class="flex flex-row gap-2">
                    <button
                        class="
                            bg-emerald-100
                            block
                            dark:bg-slate-700
                            dark:hover:bg-slate-600
                            dark:text-white
                            hover:bg-emerald-200
                            hover:shadow-none
                            p-1
                            rounded
                            shadow
                            transition
                            w-36
                        "
                            >Sign Up</button>

                    <a href="{login_route}">
                        <button
                            class="
                                border-emerald-100
                                hover:bg-emerald-100
                                border-2
                                block
                                dark:border-slate-700
                                dark:hover:bg-slate-700
                                dark:text-white
                                hover:shadow-none
                                p-1
                                rounded
                                shadow
                                transition
                                w-36
                            "
                                >Log In</button>
                    </a>
                </div>
            </form>
            "#
        )
    }
}

struct FormErrors<'a> {
    error_messages: Vec<&'a dyn Component>,
}
impl Component for FormErrors<'_> {
    fn render(&self) -> String {
        let messages =
            self.error_messages
                .iter()
                .fold(String::new(), |mut acc, msg| {
                    acc.push_str(&ErrorChip { message: *msg }.render());
                    acc
                });
        format!(
            r#"
            <div class="flex flex-col gap-2">
                {messages}
            </div>
            "#
        )
    }
}

struct ErrorChip<'a> {
    message: &'a dyn Component,
}
impl Component for ErrorChip<'_> {
    fn render(&self) -> String {
        let msg = self.message.render();
        format!(
            r#"
            <p class="self-start inline-block dark:bg-red-800 bg-red-100
                rounded p-2
            ">{msg}</p>"#
        )
    }
}

struct EmailUsed {
    email: String,
}
impl Component for EmailUsed {
    fn render(&self) -> String {
        let email = clean(&self.email);
        let reset_route = Route::PasswordReset;
        format!(
            r#"
            <span>
                An account exists using email address "{email}." Try
                <a class="cursor-pointer underline" href="{reset_route}">
                    resetting your password.
                </a>
            </span>
            "#
        )
    }
}

pub async fn get_registration_form() -> impl IntoResponse {
    Page {
        title: "Register",
        children: &PageContainer {
            children: &RegisterForm::default(),
        },
    }
    .render()
}

#[derive(Debug, Deserialize)]
pub struct RegisterFormPayload {
    username: String,
    email: String,
    password: String,
    timezone: Tz,
}

#[axum_macros::debug_handler]
pub async fn handle_registration(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<RegisterFormPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers);
    let headers = HeaderMap::new();
    let hashed_pw = pw::hash_new(&form.password);

    let stripe_id =
        stripe::create_customer(&form.username, &form.email).await?;

    let (is_username_available, is_email_available) = join![
        is_username_available(&db, &form.username),
        is_email_available(&db, &form.email)
    ];
    let is_username_available = is_username_available?;
    let is_email_available = is_email_available?;
    dbg!(is_username_available);
    dbg!(is_email_available);

    let mut errors: Vec<Box<dyn Component>> = vec![];
    if form.username.is_empty() {
        errors.push(Box::new(Span {
            content: "Username is required.".into(),
        }));
    } else if !is_username_available {
        let msg = format!(r#"Username "{}" is not available"#, form.username);
        errors.push(Box::new(Span { content: msg }));
    }
    if form.email.is_empty() {
        errors.push(Box::new(Span {
            content: "Email is required.".into(),
        }));
    } else if !is_email_available {
        errors.push(Box::new(EmailUsed {
            email: form.email.clone(),
        }));
    }

    if !errors.is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            headers,
            RegisterForm {
                username: Some(&form.username),
                email: Some(&form.email),
                password: Some(&form.password),
                errors: Some(&FormErrors {
                    error_messages: errors.iter().map(|i| i.as_ref()).collect(),
                }),
            }
            .render(),
        )
            .into_response());
    }

    let user = match session {
        Some(mut ses) => {
            if super::is_anon(&ses.username) {
                // Anon users have a long-lived session. We want to change
                // the session creation date back to "now"
                ses.created_at = Utc::now();
                let mut anon_user = User::get(
                    &db,
                    &GetUserQuery {
                        identifier: crate::db_ops::UserIdentifer::Id(
                            ses.user_id,
                        ),
                    },
                )
                .await?;
                query!(
                    "update users set salt = $1, digest = $2
                    where id = $3",
                    hashed_pw.salt,
                    hashed_pw.digest,
                    ses.user_id
                )
                .execute(&db)
                .await?;

                anon_user.username = form.username;
                anon_user.email = form.email;
                anon_user.stripe_customer_id = stripe_id;
                anon_user.save(&db).await?;

                anon_user
            } else {
                create_user(
                    &db,
                    form.username,
                    form.email,
                    &hashed_pw,
                    stripe_id,
                    stripe::SubscriptionTypes::FreeTrial(
                        config::FREE_TRIAL_DURATION,
                    ),
                )
                .await?
            }
        }
        None => {
            create_user(
                &db,
                form.username,
                form.email,
                &hashed_pw,
                stripe_id,
                stripe::SubscriptionTypes::FreeTrial(
                    config::FREE_TRIAL_DURATION,
                ),
            )
            .await?
        }
    };
    let preferences = UserPreference {
        timezone: form.timezone,
        ..Default::default()
    };
    save_user_preference(&db, user.id, &preferences).await?;
    let session = Session {
        user_id: user.id,
        username: user.username,
        created_at: Utc::now(),
    };
    let headers = session.update_headers(headers);
    let headers = htmx::redirect(headers, &Route::UserHome.as_string());
    Ok((StatusCode::OK, headers, "OK".to_string()).into_response())
}

pub async fn create_user(
    db: impl PgExecutor<'_>,
    username: String,
    email: String,
    pw: &pw::HashedPw,
    stripe_customer_id: String,
    subscription_type: stripe::SubscriptionTypes,
) -> Aresult<User> {
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

    Ok(User {
        id: query_return.id,
        created_at: query_return.created_at,
        username,
        email,
        stripe_customer_id,
        stripe_subscription_type: subscription_type,
    })
}

async fn is_username_available(
    db: impl PgExecutor<'_>,
    username: &str,
) -> Aresult<bool> {
    struct Qres {
        count: Option<i64>,
    }
    let Qres { count } = query_as!(
        Qres,
        "select count(1) from users where username = $1",
        username
    )
    .fetch_one(db)
    .await?;
    Ok(count.map(|r| r == 0).unwrap_or(false))
}

async fn is_email_available(
    db: impl PgExecutor<'_>,
    email: &str,
) -> Aresult<bool> {
    struct Qres {
        count: Option<i64>,
    }
    let Qres { count } =
        query_as!(Qres, "select count(1) from users where email = $1", email)
            .fetch_one(db)
            .await?;
    Ok(count.map(|r| r == 0).unwrap_or(false))
}
