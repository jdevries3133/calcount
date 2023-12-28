//! UI Components. [Component] trait is object-safe, allowing very nice
//! component composition patterns via Rust's dynamic dispatch features.

// In many cases, we need to do a let binding to satisfy the borrow checker
// and for some reason, clippy identifies those as unnecessary. Maybe there
// are and clippy knows more than me, maybe not.
#![allow(clippy::let_and_return)]

use super::{
    count_chat, metrics, models, preferences::UserPreference, routes::Route,
};
use ammonia::clean;
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
    pub children: Box<dyn Component + 'a>,
}

impl Component for Page<'_> {
    fn render(&self) -> String {
        // note: we'll get a compiler error here until the tailwind build
        // occurs. Make sure you use `make build` in the Makefile to get
        // both to happen together
        let tailwind = include_str!("./tailwind.generated.css");
        let htmx = Route::Htmx;
        format!(
            r#"
            <html lang="en">
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"></meta>
                    <title>{title}</title>
                    <style>
                        {tailwind}
                    </style>
                    {LIVE_RELOAD_SCRIPT}
                </head>
                <body hx-boost="true" class="dark:bg-indigo-1000 dark:text-white">
                    {body_html}
                    <script src="{htmx}"></script>
                    <script>
                        htmx.config.defaultSwapStyle = "outerHTML"
                    </script>
                </body>
            </html>
            "#,
            tailwind = tailwind,
            title = clean(self.title),
            body_html = self.children.render()
        )
    }
}

pub struct Home {
    pub trial_accounts_remaining: i64,
}
impl Component for Home {
    fn render(&self) -> String {
        let trial_acct_container = if self.trial_accounts_remaining > 0 {
            let trial_accounts = TrialAccountCounter {
                count_remaining: self.trial_accounts_remaining,
            }
            .render();
            let register_route = Route::Register;
            format!(
                r#"
                <div class="flex items-center justify-center my-12">
                    <div
                        class="bg-gradient-to-tr from-blue-300 to-indigo-300
                        rounded-full p-12 text-black"
                    >
                        <h2 class="text-xl font-bold">Create a Trial Account</h2>
                        <p class="italic text-sm">Limited availability;
                        {trial_accounts}
                        trial accounts remain!</p>
                        <p>
                            To create a 3 month free trial account, use
                            registration code
                            <span class="font-mono">"a-reddit-new-year"</span>.
                        </p>
                        <a href="{register_route}">
                            <button
                                class="
                                    bg-gradient-to-tr
                                    dark:from-blue-700
                                    dark:to-indigo-700
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
            "".into()
        };
        let login_route = Route::Login;
        let chat_demo = count_chat::ChatContainer {
            meals: &vec![],
            // See the docs for [chat_demo::handle_public_chat_demo] re the
            // choice of this time zone for the demo.
            user_timezone: Tz::US__Samoa,
            prompt: None,
            next_page: None,
            post_handler: Route::PublicChatDemo,
        }
        .render();

        let waitlist_signup = Route::WaitlistSignup;

        format!(
            r#"
            <div class="text-slate-200 m-2 sm:m-4 md:m-8">
            <h1 class="mt-2 md:mt-8 text-3xl font-extrabold">
                &#127793; Bean Count &#129752;
            </h1>
            <div class="grid md:grid-cols-3 gap-24 justfiy-center m-12">
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
                        {chat_demo}
                    </div>
                    <div class="p-4 border-8 border-slate-800">
                        <h2 class="text-xl text-center font-bold">Join the Wait List</h2>
                        <form class="flex items-start justify-center" hx-post="{waitlist_signup}">
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
                                />
                                <button class="block bg-green-800
                                hover:bg-green-700 p-2 rounded font-semibold">
                                    Submit
                                </button>
                            </div>
                        </form>
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
            </div>
            "#
        )
    }
}

pub struct TrialAccountCounter {
    count_remaining: i64,
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

pub struct LoginForm;
impl Component for LoginForm {
    fn render(&self) -> String {
        let login_route = Route::Login;
        let password_reset = Route::PasswordReset;
        format!(
            r#"
            <form class="m-2 sm:m-4 md:m-8 flex flex-col gap-2 max-w-md" hx-post="{login_route}">
                <h1 class="text-xl">Login</h1>
                <label autocomplete="username" for="identifier">
                    Username or Email
                </label>
                <input type="text" id="identifier" name="identifier" />
                <label for="passwored">Password</label>
                <input
                    autocomplete="current-password"
                    type="password"
                    id="password"
                    name="password"
                    />
                <div class="flex gap-2">
                <button class="
                    dark:bg-green-700
                    w-36
                    dark:text-white
                    dark:hover:bg-green-600
                    transition
                    shadow
                    hover:shadow-none
                    rounded
                    p-1
                    block
                ">
                        Log In
                    </button>
                    <a href="{password_reset}">
                        <button class="
                            dark:bg-yellow-700
                            w-36
                            dark:text-white
                            dark:hover:bg-yellow-600
                            transition
                            shadow
                            hover:shadow-none
                            rounded
                            p-1
                            block
                        ">
                            Reset Password
                        </button>
                    </a>
                </div>
            </form>
            "#
        )
    }
}

pub struct RegisterForm;
impl Component for RegisterForm {
    fn render(&self) -> String {
        let register_route = Route::Register;
        format!(
            r#"
            <form class="m-2 sm:m-4 md:m-8 flex flex-col gap-2 max-w-md" hx-post="{register_route}">
                <h1 class="text-xl">Register for an Account</h1>
                <label for="username">Username</label>
                <input autocomplete="username" type="text" id="username" name="username" />
                <label for="email">Email</label>
                <input type="email" id="email" name="email" />
                <label for="password">Password</label>
                <input autocomplete="current-password" type="password" id="password" name="password" />
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
                    value="a-reddit-new-year"
                />
                <input type="hidden" value="" name="timezone" id="timezone" />
                <script>
                    (() => {{
                        const el = document.getElementById("timezone");
                        el.value = Intl.DateTimeFormat().resolvedOptions().timeZone;
                    }})();
                </script>
                <button class="dark:bg-slate-700 w-36 dark:text-white dark:hover:bg-slate-600 transition shadow hover:shadow-none rounded p-1 block">Sign Up</button>
            </form>
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
        let href = self.href;
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
}
impl Component for UserHome<'_> {
    fn render(&self) -> String {
        let preferences = Route::UserPreference;
        let logout = Route::Logout;
        let username = clean(&self.user.username);
        let timezone = clean(&self.preferences.timezone.to_string());
        let macros = if self.macros.is_empty() {
            self.macros.render()
        } else {
            metrics::MacroPlaceholder {}.render()
        };
        let chat = count_chat::ChatContainer {
            meals: self.meals,
            user_timezone: self.preferences.timezone,
            prompt: None,
            next_page: Some(1),
            post_handler: Route::HandleChat,
        }
        .render();
        format!(
            r#"
            <div class="flex flex-col gap-2 m-2 sm:m-4 md:m-8">
                <div class="self-start text-black p-2 bg-blue-100 rounded-2xl">
                    <div class="flex mb-1 gap-2">
                        <p class="font-bold">Hi, {username}!</p>
                        <a class="inline" href="{logout}">
                            <button style="margin-left: auto" class="text-xs p-1 bg-red-100 hover:bg-red-200 rounded-full">
                                Log Out
                            </button>
                        </a>
                        <a class="inline" href="{preferences}">
                            <button style="margin-left: auto" class="text-xs p-1 bg-green-100 hover:bg-green-200 rounded-full">
                                View Preferences
                            </button>
                        </a>
                    </div>
                    <p class="text-xs inline-block">Timezone: {timezone}</p>
                </div>
                {macros}
                {chat}
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
