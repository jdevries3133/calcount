use super::pw;
use crate::{
    auth,
    config::{BASE_URL, RESET_TOKEN_TIMEOUT_MINUTES},
    htmx,
    prelude::*,
    smtp::send_email,
};

struct ResetRequestForm;
impl Component for ResetRequestForm {
    fn render(&self) -> String {
        let reset_route = Route::PasswordReset;
        format!(
            r#"
            <form hx-post="{reset_route}" class="flex flex-col gap-2 max-w-prose p-2 sm:p-4 md:p-8">
                <h1 class="text-xl font-extrabold">Password Reset</h1>
                <label for="email">Email Address</label>
                <p class="text-xs">
                    Input the email associated with your account
                </p>
                <input type="email" id="email" name="email" required />
                <button class="self-start dark:bg-emerald-700 dark:hover:bg-emerald-600 bg-emerald-100 hover:bg-emerald-200 rounded p-1">
                    Submit
                </button>
            </form>
            "#
        )
    }
}

struct ConfirmReset<'a> {
    email: &'a str,
}
impl Component for ConfirmReset<'_> {
    fn render(&self) -> String {
        let email = clean(self.email);
        let home = Route::Root;
        format!(
            r#"
            <div>
                <p>An password reset email was sent to {email} if an associated
                user exists.</p>
                <a class="link" href="{home}">Return to Home Page</a>
            </div>
            "#
        )
    }
}

pub async fn get_password_reset_request() -> String {
    Page {
        title: "Reset Password",
        children: &PageContainer {
            children: &ResetRequestForm {},
        },
    }
    .render()
}

#[derive(Deserialize)]
pub struct ResetPayload {
    email: String,
}

pub async fn handle_pw_reset_request(
    State(AppState { db }): State<AppState>,
    Form(ResetPayload { email }): Form<ResetPayload>,
) -> Result<impl IntoResponse, ServerError> {
    struct Qres {
        id: i32,
    }
    let uid = query_as!(Qres, "select id from users where email = $1", email)
        .fetch_optional(&db)
        .await?;
    if let Some(Qres { id }) = uid {
        // Invalidating old reset links before creating a new one feels like
        // the right move, which is also why it's enforced by our schema.
        query!("delete from password_reset_link where user_id = $1", id)
            .execute(&db)
            .await?;

        let slug = uuid::Uuid::new_v4().to_string();
        let link = Route::PasswordResetSecret(Some(slug.clone()));
        send_email(
            &email,
            "Password Reset for beancount.bot",
            &format!("Visit {BASE_URL}{link} to reset your password. This link will expire in 15 minutes."),
        )
        .await?;
        query!(
            "insert into password_reset_link (user_id, slug) values ($1, $2)",
            id,
            slug
        )
        .execute(&db)
        .await?;
    };
    Ok(ConfirmReset { email: &email }.render())
}

struct ResetForm<'a> {
    slug: &'a str,
}
impl Component for ResetForm<'_> {
    fn render(&self) -> String {
        let slug = clean(self.slug);
        let reset = Route::PasswordResetSecret(Some(slug.clone())).as_string();
        format!(
            r#"
            <form hx-post="{reset}" class="flex flex-col gap-2 max-w-prose p-2 sm:p-4 md:p-8">
                <h1 class="text-xl font-extrabold">Reset your Password</h1>
                <label for="password">New Password</label>
                <input type="password" id="password" name="password" required />
                <button class="
                    self-start
                    dark:bg-emerald-700
                    dark:hover:bg-emerald-600
                    bg-emerald-100
                    hover:bg-emerald-200
                    rounded
                    p-1
                ">
                    Save
                </button>
            </form>
            "#
        )
    }
}

/// Provides the form for submitting a password reset request
pub async fn get_password_reset_form(
    Path(slug): Path<String>,
) -> impl IntoResponse {
    Page {
        title: "Reset your Password",
        children: &PageContainer {
            children: &ResetForm { slug: &slug },
        },
    }
    .render()
}

struct ResetFailed;
impl Component for ResetFailed {
    fn render(&self) -> String {
        let retry = Route::PasswordReset;
        format!(
            r#"
            <p>
                Could not reset password.
                <a class="link" href="{retry}">Click here to try again.</a>
            </p>
            "#
        )
    }
}

#[derive(Deserialize)]
pub struct NewPassword {
    password: String,
}

/// Handles POST to the secret URL, performing the password reset if the slug
/// is valid.
pub async fn handle_password_reset(
    State(AppState { db }): State<AppState>,
    Path(slug): Path<String>,
    Form(NewPassword { password }): Form<NewPassword>,
) -> Result<impl IntoResponse, ServerError> {
    struct ResetToken {
        user_id: i32,
        username: String,
        slug: String,
        created_at: chrono::DateTime<Utc>,
    }
    let existing_token = query_as!(
        ResetToken,
        "select
            r.user_id user_id,
            r.slug slug,
            r.created_at created_at,
            u.username username
            from password_reset_link r
        join users u on u.id = r.user_id
        where slug = $1",
        slug
    )
    .fetch_optional(&db)
    .await?;

    // Now let's delete that token, as its one and only use is now consumed
    query!("delete from password_reset_link where slug = $1", slug)
        .execute(&db)
        .await?;

    let headers = HeaderMap::new();
    match existing_token {
        Some(tok) => {
            if (utc_now()
                .signed_duration_since(tok.created_at)
                .num_minutes()
                > RESET_TOKEN_TIMEOUT_MINUTES)
                || (tok.slug != slug)
            {
                Ok((headers, ResetFailed {}.render()))
            } else {
                let pw = pw::hash_new(&password);
                query!(
                    "update users set salt = $1, digest = $2
                    where id = $3",
                    pw.salt,
                    pw.digest,
                    tok.user_id
                )
                .execute(&db)
                .await?;
                let session =
                    auth::authenticate(&db, &tok.username, &password).await?;
                let homepage = Route::UserHome.as_string();
                let headers = session.update_headers(headers);
                let headers = htmx::redirect(headers, &homepage);
                Ok((headers, "OK".into()))
            }
        }
        None => Ok((headers, ResetFailed {}.render())),
    }
}
