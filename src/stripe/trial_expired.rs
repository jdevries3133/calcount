use super::create_new_subscription::get_basic_plan_checkout_session;
use crate::{
    auth::{is_anon, RegisterForm},
    prelude::*,
};

pub struct SubscriptionExpired<'a> {
    stripe_checkout_url: &'a str,
}
impl Component for SubscriptionExpired<'_> {
    fn render(&self) -> String {
        let url = clean(self.stripe_checkout_url);
        let logout = Route::Logout;
        format!(
            r#"
            <div
                class="flex flex-col bg-slate-200 m-8 p-4 items-center
                justify-center prose gap-2 rounded"
            >
                <h1>Trial Expired</h1>
                <p>
                    Your free trial has expired! Purchase a subscription on
                    stripe to regain access to your account.
                    Contact Jack DeVries at (<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>)
                    for support.
                </p>
                <!-- Note: hx-boost is disabled so that the browser can follow
                     a redirect to a different domain -->
                <a class="inline" href="{url}" hx-boost="false">
                    <button
                        style="margin-left: auto"
                        class="text-xs p-1 bg-emerald-100 hover:bg-emerald-200
                        rounded-full text-black"
                    >
                        Purchase Stripe Subscription
                    </button>
                </a>
                <a class="inline" href="{logout}">
                    <button
                        style="margin-left: auto"
                        class="text-xs p-1 bg-red-100 hover:bg-red-200
                        rounded-full text-black"
                    >
                        Log Out
                    </button>
                </a>
            </div>
            "#
        )
    }
}

struct AnonSubExpired;
impl Component for AnonSubExpired {
    fn render(&self) -> String {
        let register = RegisterForm::default().render();
        format!(
            r#"
            <div
                class="flex flex-col bg-slate-200 m-8 p-4 items-center
                justify-center prose gap-2 rounded"
            >
                <h1>Trial Expired</h1>
                <p>
                    Since your account is still anonymous, you'll need to
                    register first, then reload this page, and then you will
                    be able to proceed to stripe to make a monthly payment
                    if you'd like to continue using Bean Count.
                </p>
                {register}
            </div>
            "#
        )
    }
}
pub async fn trial_expired(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "trial expired")?;
    if is_anon(&session.username) {
        Ok(Page {
            title: "Trial Expired",
            children: &PageContainer {
                children: &AnonSubExpired {},
            },
        }
        .render())
    } else {
        let user = session.get_user(&db).await?;
        let stripe_url =
            get_basic_plan_checkout_session(&user.stripe_customer_id).await?;
        Ok(Page {
            title: "Trial Expired",
            children: &PageContainer {
                children: &SubscriptionExpired {
                    stripe_checkout_url: &stripe_url,
                },
            },
        }
        .render())
    }
}
