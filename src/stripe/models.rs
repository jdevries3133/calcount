use crate::{config, prelude::*};
use serde::Serialize;
use std::time::Duration;

/// Dang I'm realizing it would be very cool to model this as a FSM now 🤔
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SubscriptionTypes {
    /// When the stripe integration is enabled, all new users enter this state
    /// until we receive a webhook from stripe. They'll be gated from the
    /// product, and see a message that they need to go to the stripe customer
    /// portal to manage their subscription.
    Initializing,
    /// This is the $1/mo plan I intend to rollout. No one will have this
    /// until the Stripe integration goes live in production.
    Basic,
    /// This is friends and family (and me) -- free in perpetuity.
    Free,
    /// Users who have churned for any reason. Users enter this state when we
    /// receive stripe webhooks
    /// that users cancelled / disassociated / unregistered / removed /
    /// deactivated / paused / discontinued / disavowes (why, WHY, Stripe,
    /// do you have so many flavors of cancellation!?!?!).
    Unsubscribed,
    /// Trial users have an attached duration, which will be compared to
    /// [crate::models::User] `created_time`. At time of writing, this duration
    /// does not live in the database, because I am only doing 1 month trials,
    /// so we just hard-code 1 month anywhere that this variant is
    /// instantiated.
    FreeTrial(Duration),
}

impl SubscriptionTypes {
    pub fn as_int(&self) -> i32 {
        match self {
            Self::Initializing => 1,
            Self::Basic => 2,
            Self::Free => 3,
            Self::Unsubscribed => 4,
            Self::FreeTrial(_) => 5,
        }
    }
    pub fn from_int(int: i32) -> Self {
        match int {
            1 => Self::Initializing,
            2 => Self::Basic,
            3 => Self::Free,
            4 => Self::Unsubscribed,
            5 => Self::FreeTrial(config::FREE_TRIAL_DURATION),
            n => panic!("{n} is an invalid subscription type"),
        }
    }
}

/// This is my own simple and sane data-model for a stripe webhook event.
#[derive(Debug)]
pub struct StripeUpdate {
    pub stripe_customer_id: String,
    pub subscription_type: SubscriptionTypes,
}
