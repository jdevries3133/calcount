//! UI Components. [Component] trait is object-safe, allowing very nice
//! component composition patterns via Rust's dynamic dispatch features.

// In many cases, we need to do a let binding to satisfy the borrow checker
// and for some reason, clippy identifies those as unnecessary. Maybe there
// are and clippy knows more than me, maybe not.
#![allow(clippy::let_and_return)]

use super::{models, routes::Route};
use ammonia::clean;

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
            <html>
                <head>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0"></meta>
                    <title>{title}</title>
                    <style>
                        {tailwind}
                    </style>
                    {LIVE_RELOAD_SCRIPT}
                </head>
                <body hx-boost="true" class="dark:bg-indigo-1000 dark:text-white mt-2 ml-2 sm:mt-8 sm:ml-8">
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

pub struct Home;
impl Component for Home {
    fn render(&self) -> String {
        let register_route = Route::Register;
        let login_route = Route::Login;
        format!(
            r#"
            <div class="prose bg-slate-200 rounded p-2">
                <h1>calcount</h1>
                <a href="{register_route}">
                    <p>Click here to create an account use the app</p>
                </a>
                <a href="{login_route}">
                    <p>Click here to login.</p>
                </a>
            </div>
            "#
        )
    }
}

pub struct LoginForm;
impl Component for LoginForm {
    fn render(&self) -> String {
        let login_route = Route::Login;
        format!(
            r#"
            <form class="flex flex-col gap-2 max-w-md" hx-post="{login_route}">
                <h1 class="text-xl">Login</h1>
                <label autocomplete="username" for="identifier">Username or Email</label>
                <input type="text" id="identifier" name="identifier" />
                <label for="passwored">Password</label>
                <input autocomplete="current-password" type="password" id="password" name="password" />
                <button class="dark:bg-slate-700 w-36 dark:text-white dark:hover:bg-slate-600 transition shadow hover:shadow-none rounded p-1 block">Log In</button>
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
            <form class="flex flex-col gap-2 max-w-md" hx-post="{register_route}">
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
                <input type="text" id="registration_key" name="registration_key" />
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
}
impl Component for UserHome<'_> {
    fn render(&self) -> String {
        let username = clean(&self.user.username);
        format!(
            r#"
            <h1>Hi {username}</h1>
            <p>Welcome to calcount.</p>
            "#
        )
    }
}
