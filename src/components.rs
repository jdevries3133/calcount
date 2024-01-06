//! UI Components. [Component] trait is object-safe, allowing very nice
//! component composition patterns via Rust's dynamic dispatch features.

// In many cases, we need to do a let binding to satisfy the borrow checker
// and for some reason, clippy identifies those as unnecessary. Maybe there
// are and clippy knows more than me, maybe not.
#![allow(clippy::let_and_return)]

use super::{count_chat, metrics, models, prelude::*, stripe, timeutils};
use ammonia::clean;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;

#[cfg(feature = "live_reload")]
const LIVE_RELOAD_SCRIPT: &str = r#"<script>
    (async () => {
        while (true) {
            try {
                await fetch('/ping?poll_interval_secs=60');
            } catch (e) {
                console.log("hup from ping; let's live-reload");
                const el = document.createElement('p');
                el.innerText = "Reloading...";
                el.classList.add("bg-yellow-100");
                el.classList.add("p-2");
                el.classList.add("rounded");
                el.classList.add("w-full");
                el.classList.add("dark:text-black");
                document.body.insertBefore(el, document.body.firstChild);
                setInterval(async () => {
                    setTimeout(() => {
                        // At some point, a compiler error may be preventing
                        // the server from coming back
                        el.innerText = "Reload taking longer than usual; check for a compiler error";
                    }, 2000);
                    // Now the server is down, we'll fast-poll it (trying to
                    // get an immediate response), and reload the page when it
                    // comes back
                    try {
                        await fetch('/ping?poll_interval_secs=0');
                        window.location.reload()
                    } catch (e) {}
                }, 100);
                break;
            }
        }
    })();
</script>"#;

#[cfg(not(feature = "live_reload"))]
const LIVE_RELOAD_SCRIPT: &str = "";

pub trait Component {
    /// Render the component to a HTML string. By convention, the
    /// implementation should sanitize all string properties at render-time
    fn render(&self) -> String;
}

impl std::fmt::Display for dyn Component {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.render())
    }
}

pub struct Page<'a> {
    pub title: &'a str,
    pub children: &'a dyn Component,
}

impl Component for Page<'_> {
    fn render(&self) -> String {
        // note: we'll get a compiler error here until the tailwind build
        // occurs. Make sure you use `make build` in the Makefile to get
        // both to happen together
        let tailwind = include_str!("./tailwind.generated.css");
        let htmx = Route::Htmx;
        let apple_icon = Route::StaticAppleIcon;
        let manifest = Route::StaticManifest;
        format!(
            r##"<!DOCTYPE html>
            <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"></meta>
                    <meta name="theme-color" content="#BBF7D0"/>
                    <meta name="description" content="ChatGPT-powered calorie counter" />
                    <title>{title}</title>
                    <style>
                        {tailwind}
                    </style>
                    {LIVE_RELOAD_SCRIPT}
                    <link rel="manifest" href="{manifest}" />
                    <link rel="apple-touch-icon" href="{apple_icon}">
                </head>
                <body hx-boost="true">
                    {body_html}
                    <script src="{htmx}"></script>
                    <script>
                        htmx.config.defaultSwapStyle = "outerHTML"
                    </script>
                </body>
            </html>
            "##,
            tailwind = tailwind,
            title = clean(self.title),
            body_html = self.children.render()
        )
    }
}

pub struct PageContainer<'a> {
    pub children: &'a dyn Component,
}
impl Component for PageContainer<'_> {
    fn render(&self) -> String {
        let children = self.children.render();
        let privacy = Route::PrivacyPolicy;
        let tos = Route::TermsOfService;
        let home = Route::UserHome;
        format!(
            r#"
            <div
                class="p-2 sm:p-4 md:p-8 bg-teal-50 dark:bg-indigo-1000
                dark:text-slate-200 min-h-[100vh]"
            >
                {children}
                <div class="flex items-center justify-center gap-2 mt-4">
                    <a class="link" href="{privacy}">Privacy Policy</a>
                    <a class="link" href="{tos}">Terms of Service</a>
                    <a class="link" href="{home}">Home</a>
                </div>
            </div>
            "#
        )
    }
}

pub struct Home {
    pub trial_accounts_remaining: usize,
}
impl Component for Home {
    fn render(&self) -> String {
        let trial_acct_container = if self.trial_accounts_remaining > 0 {
            let trial_accounts = TrialAccountCounter {
                count_remaining: self.trial_accounts_remaining,
            }
            .render();
            let register_route = Route::Register;
            let s = if self.trial_accounts_remaining > 1 {
                "s"
            } else {
                ""
            };
            let snt = if self.trial_accounts_remaining > 1 {
                ""
            } else {
                "s"
            };
            format!(
                r#"
                <div class="flex items-center justify-center my-12">
                    <div
                        class="bg-gradient-to-tr from-blue-300 to-indigo-300
                        rounded-full p-12 text-black"
                    >
                        <h2 class="text-xl font-bold">Create a Trial Account</h2>
                        <p class="italic text-sm">
                            {trial_accounts} free trial account{s} remain{snt}!
                        </p>
                        <p>
                            To create a 30-day free trial account, use
                            registration code
                            <span class="font-mono">"a-reddit-new-year"</span>.
                        </p>
                        <p class="text-xs">Price will be $5/mo</p>
                        <a href="{register_route}">
                            <button
                                class="
                                    bg-gradient-to-tr
                                    from-blue-700
                                    to-indigo-700
                                    from-blue-100
                                    to-indigo-200
                                    p-2
                                    rounded
                                    shadow-md
                                    hover:shadow-sm
                                    dark:shadow-purple-200
                                    text-xl
                                    font-extrabold
                                    text-white
                                    my-4
                                "
                            >Sign Up</button>
                        </a>
                    </div>
                </div>
                "#
            )
        } else {
            r#"
            <div class="flex items-center justify-center my-12">
                <div
                    class="bg-gradient-to-tr from-blue-300 to-indigo-300
                    rounded-full p-12 text-black"
                >
                    <h2 class="text-xl font-bold">All Trial Accounts Taken!</h2>
                    <p class="max-w-xs">
                        Uh oh! Bad news for you but good news for us: 200 trial
                        users already registered. Join the wait list to be
                        notified when we're ready for you to make an account.
                    </p>
                    <p class="text-xs">Price will be $5/mo</p>
                </div>
            </div>
            "#
            .into()
        };
        let login_route = Route::Login;
        let waitlist_signup = Route::WaitlistSignup;
        let chat_demo = count_chat::ChatDemo {
            prefill_prompt: None,
        }
        .render();

        format!(
            r#"
            <h1 class="mt-2 md:mt-8 text-3xl font-extrabold">
                &#127793; Bean Count &#129752;
            </h1>
            <div
                class="text-teal-50 dark:text-slate-200 grid md:grid-cols-3
                gap-24 justfiy-center m-12"
            >
                <div
                    class="bg-blue-800 rounded p-2 inline-block my-2 flex
                    items-center text-lg font-semibold text-center"
                >
                    Bean Count is an AI-powered  calorie counter, making calorie
                    counting easy, effortless, and fun!
                </div>
                <div
                    class="bg-indigo-800 rounded p-2 inline-block my-2 flex
                    items-center text-lg font-semibold text-center"
                >
                    Use natural language to ask about food, and get back quick
                    calorie estimates.
                </div>
                <div
                    class="bg-purple-800 rounded p-2 inline-block my-2 flex
                    items-center text-lg font-semibold text-center"
                >
                    Keep track of total calories and grams of macros (carbs, fat,
                    and protein) as they accumulate throughout the day.
                </div>
            </div>
            {trial_acct_container}
            <div class="flex justify-center">
                <div class="grid md:grid-cols-2 gap-3 max-w-[1200px]">
                    <div class="p-4 border-8 border-slate-800">
                        <h2 class="text-xl font-bold text-center">Try it Out</h2>
                        <p class="text-center text-sm max-w-md">
                            With Bean Count, you can describe your food using
                            natural language. Since you don't need to measure
                            or lookup precise calorie information, calorie
                            counting becomes easier than ever before!
                        </p>
                        {chat_demo}
                    </div>
                    <div class="p-4 border-8 border-slate-800">
                        <h2 class="text-xl text-center font-bold">
                            Join the Wait List
                        </h2>
                        <form
                            class="flex items-start justify-center"
                            hx-post="{waitlist_signup}"
                        >
                            <div class="flex flex-col gap-2">
                                <label class="block" for="email">
                                    Email Address
                                </label>
                                <input
                                    class="block"
                                    type="email"
                                    name="email"
                                    id="email"
                                    placeholder="Your Email"
                                    required
                                />
                                <button class="block dark:bg-green-800
                                dark:hover:bg-green-700 bg-green-200
                                hover:bg-green-300 p-2 rounded font-semibold">
                                    Submit
                                </button>
                            </div>
                        </form>
                        <p class="text-center text-xs">
                            Join the waitlist to find out when we're ready
                            for unrestricted sign-ups. These emails will
                            never be shared with a 3rd party, and will only
                            be used to provide important updates about this
                            product, and you can be removed from this list
                            at anytime.
                        </p>
                    </div>
                </div>
            </div>
            <div class="flex items-center justify-center">
                <div class="bg-indigo-50 dark:bg-indigo-900 border-2
                    border-indigo-800 inline-flex p-6 rounded-full
                    items-center gap-3 mt-2"
                >
                    <p>Have an account?</p>
                    <a href="{login_route}">
                        <button
                            class="border-2 border-slate-800 rounded p-2"
                        >Log In</button>
                    </a>
                </div>
            </div>
            "#
        )
    }
}

pub struct TrialAccountCounter {
    count_remaining: usize,
}
impl Component for TrialAccountCounter {
    fn render(&self) -> String {
        let count_remaining = self.count_remaining;
        format!(
            r#"
            <span hx-trigger="load delay:5s">{count_remaining}</span>
            "#
        )
    }
}

pub struct ExternalLink<'a> {
    pub href: &'a str,
    pub children: Box<dyn Component>,
}
impl Component for ExternalLink<'_> {
    fn render(&self) -> String {
        let children = self.children.render();
        let href = clean(self.href);
        format!(
            r#"
            <a href={href}>
                {children}
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-3 h-3 inline">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 19.5l15-15m0 0H8.25m11.25 0v11.25" />
                </svg>
            </a>
            "#
        )
    }
}

pub struct UserHome<'a> {
    pub user: &'a models::User,
    pub preferences: UserPreference,
    pub meals: &'a Vec<count_chat::Meal>,
    pub macros: &'a metrics::Macros,
    pub subscription_type: SubscriptionTypes,
    pub caloric_intake_goal: Option<i32>,
}
impl Component for UserHome<'_> {
    fn render(&self) -> String {
        let macros = if self.macros.is_empty() {
            self.macros.render_status(self.caloric_intake_goal)
        } else {
            metrics::MacroPlaceholder {}.render()
        };
        let profile = ProfileChip {
            username: &self.user.username,
            timezone: &self.preferences.timezone,
            subscription_type: self.subscription_type,
            user_created_time: self.user.created_at,
        }
        .render();
        let chat = count_chat::ChatContainer {
            meals: self.meals,
            user_timezone: self.preferences.timezone,
            prompt: None,
            next_page: 1,
            post_request_handler: Route::HandleChat,
        }
        .render();
        format!(
            r#"
            <div class="flex flex-col gap-2">
                {profile}
                {macros}
                {chat}
            </div>
            "#
        )
    }
}

struct ProfileChip<'a> {
    username: &'a str,
    timezone: &'a Tz,
    user_created_time: DateTime<Utc>,
    subscription_type: SubscriptionTypes,
}
impl Component for ProfileChip<'_> {
    fn render(&self) -> String {
        let username = clean(self.username);
        let timezone = self.timezone;
        let logout = Route::Logout;
        let preferences = Route::UserPreference;
        let trial_warning = if let SubscriptionTypes::FreeTrial(duration) =
            self.subscription_type
        {
            let cnt_remaining_days = timeutils::as_days(
                duration
                    .checked_sub(
                        Utc::now()
                            .signed_duration_since(self.user_created_time)
                            .to_std()
                            .unwrap_or_default(),
                    )
                    .unwrap_or_default(),
            );
            if cnt_remaining_days == 0 {
                r#"
                <p
                    class="text-black text-xs inline-block bg-yellow-100 p-1 rounded-lg my-2"
                >
                    <span class="font-semibold">Free Trial Ends Tomorrow!</span>
                </p>
                "#.to_string()
            } else {
                format!(
                    r#"
                    <p
                        class="text-black text-xs inline-block bg-yellow-100 p-1 rounded-lg my-2"
                    >
                        <span class="font-semibold">{cnt_remaining_days}</span>
                        free trial days remaining; price will be $5/mo
                    </p>
                    "#
                )
            }
        } else {
            "".into()
        };
        let billing_portal_button = stripe::PortalLink {
            subscription_type: self.subscription_type,
        }
        .render();
        format!(
            r#"
            <div class="self-start p-2 bg-blue-100 dark:bg-blue-800 rounded-2xl">
                <div class="flex mb-1 gap-2">
                    <p class="font-bold">Hi, {username}!</p>
                    <a class="inline" href="{logout}">
                        <button
                            style="margin-left: auto"
                            class="text-xs p-1 bg-red-100 hover:bg-red-200
                            rounded-full text-black"
                        >
                            Log Out
                        </button>
                    </a>
                    <a class="inline" href="{preferences}">
                        <button
                            style="margin-left: auto"
                            class="text-xs p-1 bg-blue-100 hover:bg-blue-200
                            rounded-full text-black"
                        >
                            Goals & Preferences
                        </button>
                    </a>
                    {billing_portal_button}
                </div>
                <p class="text-xs inline-block">Timezone: {timezone}</p>
                {trial_warning}
            </div>
            "#
        )
    }
}

pub struct Saved<'a> {
    pub message: &'a str,
}
impl Component for Saved<'_> {
    fn render(&self) -> String {
        let void = Route::Void;
        let message = clean(self.message);
        format!(
            r##"
            <div
                hx-get="{void}"
                hx-trigger="load delay:2s"
                class="my-2"
                >
                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="inline bg-green-800 p-2 rounded-full w-8 h-8">
                  <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                </svg>
                {message}
                <script>
                    setTimeout(() => {{
                        const iconElement = document.querySelector("#sort-icon");
                        iconElement.classList.remove('text-black');
                        iconElement.classList.remove('bg-yellow-100');
                        htmx.trigger('body', 'toggle-sort-toolbar');
                    }}, 2000);
                </script>
            </div>
            "##
        )
    }
}
