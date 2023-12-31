use super::pw;
use crate::{
    config, htmx, models::IdCreatedAt, preferences::save_user_preference,
    prelude::*, stripe,
};
use std::collections::HashSet;

pub struct RegisterForm {
    pub should_prefill_registration_key: bool,
}
impl Component for RegisterForm {
    fn render(&self) -> String {
        let key = if self.should_prefill_registration_key {
            "a-reddit-new-year"
        } else {
            ""
        };
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
                <label for="registration_key">Registration Key</label>
                <p class="text-sm dark:text-slate-100">
                    A registration key from the developer is required to create
                    an account at this time.
                </p>
                <input
                    class="font-mono"
                    id="registration_key"
                    name="registration_key"
                    type="text"
                    value="{key}"
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

pub async fn get_registration_form(
    State(AppState { db }): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let account_total =
        query!("select 1 count from users").fetch_all(&db).await?;
    let trial_accounts_remaining = config::MAX_ACCOUNT_LIMIT
        .checked_sub(account_total.len())
        .unwrap_or_default();
    Ok(Page {
        title: "Register",
        children: &PageContainer {
            children: &RegisterForm {
                should_prefill_registration_key: trial_accounts_remaining > 0,
            },
        },
    }
    .render())
}

async fn maybe_revoke_reddit_registration(db: &PgPool) -> Aresult<()> {
    let result = query!("select 1 count from users").fetch_all(db).await?;
    if result.len() >= config::MAX_ACCOUNT_LIMIT {
        query!("delete from registration_key where key = 'a-reddit-new-year'")
            .execute(db)
            .await?;
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RegisterFormPayload {
    username: String,
    email: String,
    password: String,
    registration_key: String,
    timezone: Tz,
}

pub async fn handle_registration(
    State(AppState { db }): State<AppState>,
    Form(form): Form<RegisterFormPayload>,
) -> Result<impl IntoResponse, ServerError> {
    let headers = HeaderMap::new();
    let registration_keys = get_registraton_keys(&db).await?;
    let user_key = form.registration_key;
    if !registration_keys.contains(&user_key) {
        println!("{user_key} is not a known registration key");
        let register_route = Route::Register;
        return Ok((
            headers,
            format!(
                r#"<p hx-trigger="load delay:1s" hx-get="{register_route}">Wrong registration key.</p>"#
            ),
        ));
    };
    let hashed_pw = pw::hash_new(&form.password);

    let stripe_id =
        stripe::create_customer(&form.username, &form.email).await?;
    let payment_portal_url = stripe::get_billing_portal_url(&stripe_id).await?;

    let user = create_user(
        &db,
        form.username,
        form.email,
        &hashed_pw,
        stripe_id,
        stripe::SubscriptionTypes::FreeTrial(config::FREE_TRIAL_DURATION),
    )
    .await?;
    maybe_revoke_reddit_registration(&db).await?;
    let preferences = UserPreference {
        timezone: form.timezone,
        caloric_intake_goal: None,
    };
    save_user_preference(&db, &user, &preferences).await?;
    let session = Session {
        user,
        preferences,
        created_at: Utc::now(),
    };
    let headers = session.update_headers(headers);
    let headers = htmx::redirect(headers, &payment_portal_url);
    Ok((headers, "OK".to_string()))
}

pub async fn get_registraton_keys(db: &PgPool) -> Aresult<HashSet<String>> {
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

pub async fn create_user(
    db: &PgPool,
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
