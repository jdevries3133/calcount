use super::create_new_subscription::get_basic_plan_checkout_session;
use crate::{auth::is_anon, components, prelude::*};

pub struct SubscriptionExpired<'a> {
    checkout_url: &'a str,
}
impl Component for SubscriptionExpired<'_> {
    fn render(&self) -> String {
        let portal_url = Route::GotoStripePortal;
        let checkout_url = clean(self.checkout_url);
        let logout = Route::Logout;
        format!(
            r#"
            <div
                class="flex flex-col bg-slate-200 m-8 p-4 items-center
                justify-center prose gap-2 rounded"
            >
                <h1>Subscription is Expired</h1>
                <p>
                    Your subscription is not active. Don't worry, all your
                    Bean Count data is still here; you just need to visit
                    Stripe to see what is wrong and resume your subscription!
                    Contact Jack DeVries at (<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>)
                    for support.
                </p>
                <!-- Note: hx-boost is disabled so that the browser can follow
                     a redirect to a different domain -->
                <a class="inline" href="{portal_url}" hx-boost="false">
                    <button
                        style="margin-left: auto"
                        class="text-xs p-1 bg-green-100 hover:bg-green-200
                        rounded-full text-black"
                    >
                        Manage Existing Subscription via Stripe Customer Portal
                    </button>
                </a>
                <div class="border-blue-300 border-2 rounded-xl p-4 m-4">
                    <!-- Note: hx-boost is disabled so that the browser can follow
                         a redirect to a different domain -->
                    <a class="inline" href="{checkout_url}" hx-boost="false">
                        <button
                            style="margin-left: auto"
                            class="text-xs p-1 bg-blue-100 hover:bg-blue-200
                            rounded-full text-black"
                        >
                            Purchase New Stripe Subscription
                        </button>
                    </a>
                    <p class="text-xs">
                        If no &quot;resume subscription,&quot; option appears
                        when you visit the customer portal, it meas that you
                        need to use this option instead to purchase a new
                        subscription; presumably because your previous
                        subscription was cancelled, and therefore cannot be
                        resumed.
                    </p>
                </div>
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

pub async fn subscription_ended(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "subscription ended")?;
    // It should be impossible for this to happen, because anonymous users
    // cannot create subscriptions in the first place.
    if is_anon(&session.username) {
        let uid = session.user_id;
        println!("Warning: anonymous (id = {uid}) user is visiting subscription_ended; this probably won't work")
    }
    let user = session.get_user(&db).await?;
    let checkout_url =
        get_basic_plan_checkout_session(&user.stripe_customer_id).await?;
    Ok(components::Page {
        title: "Bean Count Subscription Expired",
        children: &components::PageContainer {
            children: &SubscriptionExpired {
                checkout_url: &checkout_url,
            },
        },
    }
    .render())
}
