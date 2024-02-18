use super::pw;
use crate::{
    config,
    db_ops::{GetModel, GetUserQuery, SaveModel},
    htmx,
    models::{IdCreatedAt, User},
    preferences::save_user_preference,
    prelude::*,
    stripe,
};

pub struct RegisterForm {}
impl Component for RegisterForm {
    fn render(&self) -> String {
        let register_route = Route::Register;
        format!(
            r#"
            <form class="flex flex-col gap-2 max-w-md" hx-post="{register_route}">
                <h1 class="text-xl">Register for a Bean Count Account</h1>
                <label for="username">Username</label>
                <input autocomplete="username" type="text" id="username" name="username" />
                <label for="email">Email</label>
                <input type="email" id="email" name="email" />
                <label for="password">Password</label>
                <input
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
                <button
                    class="
                        bg-green-100
                        block
                        dark:bg-slate-700
                        dark:hover:bg-slate-600
                        dark:text-white
                        hover:bg-green-200
                        hover:shadow-none
                        p-1
                        rounded
                        shadow
                        transition
                        w-36
                    ">
                        Sign Up
                    </button>
            </form>
            "#
        )
    }
}

pub async fn get_registration_form() -> impl IntoResponse {
    Page {
        title: "Register",
        children: &PageContainer {
            children: &RegisterForm {},
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
    Ok((headers, "OK".to_string()))
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
