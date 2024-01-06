use super::create_new_subscription::get_basic_plan_checkout_session;
use crate::prelude::*;

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
                <a class="inline" href="{url}" hx-boost="false">
                    <button
                        style="margin-left: auto"
                        class="text-xs p-1 bg-green-100 hover:bg-green-200
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
pub async fn trial_expired(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "trial expired")?;
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
