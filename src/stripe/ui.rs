use crate::prelude::*;

pub struct PortalLink {
    pub subscription_type: SubscriptionTypes,
}
impl Component for PortalLink {
    fn render(&self) -> String {
        match self.subscription_type {
            SubscriptionTypes::Basic | SubscriptionTypes::Unsubscribed => {
                let url = Route::GotoStripePortal;
                // Note: we need to disable hx-boost because the browser needs
                // to follow the redirect to a new origin.
                format!(
                    r#"
                    <a class="inline" href="{url}" hx-boost="false">
                        <button
                            style="margin-left: auto"
                            class="text-xs p-1 bg-green-100 hover:bg-green-200
                            rounded-full text-black"
                        >
                            Manage Subscription via Stripe
                        </button>
                    </a>
                    "#
                )
            }
            _ => "".into(),
        }
    }
}
