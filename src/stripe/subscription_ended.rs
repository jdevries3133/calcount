use super::ui::PortalLink;
use crate::{components, prelude::*};

pub struct SubscriptionExpired {
    subscription_type: SubscriptionTypes,
}
impl Component for SubscriptionExpired {
    fn render(&self) -> String {
        let stripe_link = PortalLink {
            subscription_type: self.subscription_type,
        }
        .render();
        let logout = Route::Logout;
        format!(
            r#"
            <div
                class="flex flex-col bg-slate-200 m-8 p-4 items-center
                justify-center prose gap-2 rounded"
            >
                <h1>Subscription is Expired</h1>
                <p>
                    We received a notification from Stripe that your
                    subscription has been canceled. Don't worry, all your
                    Bean Count data is still here; you just need to visit
                    Stripe to see what is wrong and resume your subscription!
                    Contact Jack DeVries at (<a href="mailto:jdevries3133@gmail.com">jdevries3133@gmail.com</a>)
                    for support.
                </p>
            {stripe_link}
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
    let user = session.get_user(&db).await?;
    Ok(components::Page {
        title: "Bean Count Subscription Expired",
        children: &components::PageContainer {
            children: &SubscriptionExpired {
                subscription_type: user.stripe_subscription_type,
            },
        },
    }
    .render())
}
