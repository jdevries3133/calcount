use std::time::Duration;

/// This is for all authentication sessions; users will need to log in again
/// every 7 days, since we basically have JWT authentication.
pub const SESSION_EXPIRY_TIME_DAYS: i64 = 7;

/// Password reset links will expire after 15 minutes.
pub const RESET_TOKEN_TIMEOUT_MINUTES: i64 = 15;

#[cfg(not(feature = "localhost_base_url"))]
pub const BASE_URL: &str = "https://beancount.bot";

#[cfg(feature = "localhost_base_url")]
pub const BASE_URL: &str = "http://localhost:8000";

/// Messages which exceed this length limit will not be forwarded to OpenAI,
/// and will instead provide the user with an error message.
pub const CHAT_MAX_LEN: usize = 200;

/// We advertise a 30-day free trial, but we'll make it 31 days in duration,
/// since the clock starts as soon as registration happens and we want to
/// ensure that everyone gets 30 full days. Consider -- in the moment after
/// registration, the user would be left with only 29 full days remaining,
/// so we don't want a UX where someone registers and then the trial counter
/// shows "29 days remaining."
pub const FREE_TRIAL_DURATION: Duration =
    Duration::from_secs(60 * 60 * 24 * 31);

/// For initial launch, we're going to cap out at 200 registrations. When this
/// threshold is passed, 2 things will happen:
///
/// 1. `a-reddit-new-year` registration key will be revoked
/// 2. landing page will update, making the mailing list become the priary focus
pub const MAX_ACCOUNT_LIMIT: usize = 205;

#[cfg(feature = "use_stripe_test_instance")]
pub const BASIC_PLAN_STRIPE_ID: &str = "price_1OTyEXBhmccJFhTPvs01VoJf";

#[cfg(not(feature = "use_stripe_test_instance"))]
pub const BASIC_PLAN_STRIPE_ID: &str = "price_1OVybrBhmccJFhTPiLUXZm1P";

/// This is just Jack and Kate while calorie balancing is turbo-jank.
pub fn enable_calorie_balancing(user_id: i32) -> bool {
    user_id == 1 || user_id == 6 || user_id == 12
}
