//! The core calorie counting feature (models, components, and controllers
//! are colocated here).

use super::{llm_parse_response::ParserResult, openai::OpenAI};
use crate::{
    auth::is_anon, chrono_utils::is_before_today, client_events,
    components::AnonWarning, config, prelude::*,
};
use axum::extract::Query;
use futures::join;

const MEAL_PAGE_SIZE: u8 = 50;

#[derive(Debug)]
pub struct Meal {
    pub id: i32,
    pub info: MealInfo,
}

#[derive(Debug, Deserialize)]
pub struct MealInfo {
    pub calories: i32,
    pub protein_grams: i32,
    pub carbohydrates_grams: i32,
    pub fat_grams: i32,
    pub meal_name: String,
    pub created_at: DateTime<Utc>,
}

struct Void;
impl Component for Void {
    fn render(&self) -> String {
        "".into()
    }
}
impl Component for Meal {
    fn render(&self) -> String {
        MealCard {
            meal_id: Some(self.id),
            info: &self.info,
            actions: Some(&Void {}),
            rendering_behavior: RenderingBehavior::RenderAsToday,
            show_ai_warning: false,
        }
        .render()
    }
}

pub struct Chat<'a> {
    pub meals: &'a Vec<Meal>,
    pub prompt: Option<&'a str>,
    pub user_timezone: Tz,
    pub next_page: i64,
    pub post_request_handler: Route,
    /// If set, we'll render a banner nagging the user to register, and
    /// warning them of their tenuous state!
    pub is_anonymous: bool,
}
impl Component for Chat<'_> {
    fn render(&self) -> String {
        let prev_meals = PreviousMeals {
            meals: self.meals,
            user_timezone: self.user_timezone,
            next_page: self.next_page,
        };
        let chat = ChatUI {
            post_request_handler: &self.post_request_handler,
            prefill_prompt: self.prompt,
            children: Some(&prev_meals),
        }
        .render();
        let anon_warning = if self.is_anonymous && self.meals.len() >= 3 {
            AnonWarning {}.render()
        } else {
            "".into()
        };
        format!("{chat}{anon_warning}")
    }
}

struct PreviousMeals<'a> {
    meals: &'a Vec<Meal>,
    user_timezone: Tz,
    next_page: i64,
}
impl Component for PreviousMeals<'_> {
    fn render(&self) -> String {
        let meals = MealSet {
            meals: &self.meals[..],
            user_timezone: self.user_timezone,
            next_page: self.next_page,
            show_ai_warning: false,
        }
        .render();
        let is_any_meal_during_today = self
            .meals
            .iter()
            .any(|m| !is_before_today(&m.info.created_at, self.user_timezone));
        let meal_header = if self.meals.is_empty() {
            ""
        } else if is_any_meal_during_today {
            // Pushing the top up by just 1px hides the text from revealing
            // itself behind the top of this sticky header as the user scrolls
            // through the container; weird browser behavior, weird fix.
            r#"<h2 class="
                    sticky
                    top-[-1px]
                    bg-zinc-50
                    dark:bg-slate-900
                    rounded
                    p-2
                    text-xl
                    font-bold
                ">
                    Today's Food
                </h2>"#
        } else {
            r#"<h2 class="
                    sticky
                    top-[-1px]
                    bg-zinc-50
                    dark:bg-slate-900
                    rounded
                    p-2
                    text-xl
                    font-bold
                ">
                    Previously Saved Items
                </h2>"#
        };
        let refresh_meals_href = format!("{}?page=0", Route::ListMeals);
        format!(
            r#"
            <div
                class="flex flex-col gap-2 md:max-h-[70vh] md:overflow-y-auto"
            >
                {meal_header}
                <div
                    hx-get="{refresh_meals_href}"
                    hx-swap="innerHTML"
                    hx-trigger="reload-meals from:body"
                    class="flex flex-col gap-2"
                >
                {meals}
                </div>
            </div>
            "#
        )
    }
}

pub struct ChatUI<'a> {
    pub post_request_handler: &'a Route,
    pub prefill_prompt: Option<&'a str>,
    /// If provided, these are inserted at the end of the chat container. This
    /// is used on the user home page for injecting the list of previous meals.
    pub children: Option<&'a dyn Component>,
}
impl Component for ChatUI<'_> {
    fn render(&self) -> String {
        let handler = &self.post_request_handler;
        let prompt =
            encode_quotes(&clean(self.prefill_prompt.unwrap_or_default()));
        let children = self.children.map_or("".to_string(), |c| c.render());
        format!(
            r#"
            <div id="cal-chat-container" class="sm:flex sm:items-center sm:justify-center">
                <div class="
                    bg-zinc-50
                    border-2
                    border-black
                    dark:bg-indigo-1000
                    md:dark:bg-blue-950
                    md:dark:border-white
                    rounded
                    p-2
                ">
                    <h1
                        class="
                            border-b-2
                            border-slate-600
                            mb-2
                            border-black
                            dark:border-slate-200
                            md:dark:border-black
                            serif
                            font-extrabold
                            text-3xl
                        ">
                            Bean Count &#129752;
                        </h1>
                    <div class="md:flex md:gap-3">
                        <div>
                            <form
                                class="flex flex-col gap-2"
                                hx-post="{handler}"
                            >
                                <label for="chat">
                                    <h2
                                        class="text-xl bold"
                                    >Describe what you're eating</h2>
                                </label>
                                <input
                                    class="rounded"
                                    autocomplete="one-time-code"
                                    type="text"
                                    id="chat"
                                    name="chat"
                                    placeholder="I am eating..."
                                    value="{prompt}"
                                    required
                                    tabindex="1"
                                />
                                <button tabindex="2" class="
                                    bg-green-100
                                    dark:bg-green-800
                                    dark:hover:bg-green-700
                                    hover:bg-green-200
                                    p-2
                                    rounded
                                ">
                                    Count It
                                </button>
                            </form>
                        </div>
                        {children}
                    </div>
                </div>
            </div>
            "#
        )
    }
}

struct NewMealOptions<'a> {
    info: &'a MealInfo,
}
impl Component for NewMealOptions<'_> {
    fn render(&self) -> String {
        let retry_route = Route::ChatForm;
        let save_route = Route::SaveMeal;
        let prev_day_route = Route::PreviousDayMeal;
        let calories = self.info.calories;
        let protein = self.info.protein_grams;
        let carbs = self.info.carbohydrates_grams;
        let fat = self.info.fat_grams;
        let created_at = self.info.created_at;
        let meal_name = encode_quotes(&clean(&self.info.meal_name));
        format!(
            r##"
            <form hx-post="{prev_day_route}" hx-target="#cal-chat-container">
                <input type="hidden" value="{meal_name}" name="meal_name" />
                <input type="hidden" value="{calories}" name="calories" />
                <input type="hidden" value="{protein}" name="protein_grams" />
                <input type="hidden" value="{carbs}" name="carbohydrates_grams" />
                <input type="hidden" value="{fat}" name="fat_grams" />
                <input type="hidden" value="{created_at}" name="created_at" />
                <button
                    class="bg-indigo-100 p-1 rounded shadow hover:bg-indigo-200"
                    tabindex="3"
                >Set Date</button>
            </form>
            <form hx-post="{retry_route}" hx-target="#cal-chat-container">
                <input type="hidden" value="{meal_name}" name="meal_name" />
                <button
                    class="bg-red-100 p-1 rounded shadow hover:bg-red-200"
                    tabindex="2"
                >Try Again</button>
            </form>
            <form hx-post="{save_route}" hx-target="#cal-chat-container">
                <input type="hidden" value="{meal_name}" name="meal_name" />
                <input type="hidden" value="{calories}" name="calories" />
                <input type="hidden" value="{protein}" name="protein_grams" />
                <input type="hidden" value="{carbs}" name="carbohydrates_grams" />
                <input type="hidden" value="{fat}" name="fat_grams" />
                <input type="hidden" value="{created_at}" name="created_at" />
                <button
                    class="bg-blue-100 p-1 rounded shadow hover:bg-blue-200"
                    tabindex="1"
                    id="add-meal"
                >Add</button>
                <script>
                    // place focus on the add-meal button (above)
                    document.getElementById('add-meal').focus()
                </script>
            </form>
            "##
        )
    }
}

pub enum RenderingBehavior {
    UseTimezone(Tz),
    RenderAsToday,
}

pub struct MealCard<'a> {
    pub info: &'a MealInfo,
    pub meal_id: Option<i32>,
    pub actions: Option<&'a dyn Component>,
    pub rendering_behavior: RenderingBehavior,
    pub show_ai_warning: bool,
}
impl Component for MealCard<'_> {
    fn render(&self) -> String {
        let is_meal_before_today = match self.rendering_behavior {
            RenderingBehavior::UseTimezone(tz) => {
                is_before_today(&self.info.created_at, tz)
            }
            RenderingBehavior::RenderAsToday => false,
        };
        let meal_name = clean(&self.info.meal_name);
        let calories = self.info.calories;
        let protein = self.info.protein_grams;
        let carbs = self.info.carbohydrates_grams;
        let fat = self.info.fat_grams;
        let actions = match &self.actions {
            Some(action) => action.render(),
            None => match self.meal_id {
                Some(id) => {
                    let delete_href = Route::DeleteMeal(Some(id));
                    let add_to_today_href = Route::AddMealToToday(Some(id));
                    let add_to_today_button = if is_meal_before_today {
                        format!(
                            r#"
                            <button
                                hx-post="{add_to_today_href}"
                                hx-target="closest div[data-name='meal-card']"
                                class="
                                    align-self-right
                                    bg-green-100
                                    hover:bg-green-200
                                    rounded
                                    p-1
                                    dark:text-black
                                ">
                                Add to Today
                            </button>
                            "#
                        )
                    } else {
                        format!(
                            r#"
                            <button
                                hx-post="{add_to_today_href}"
                                hx-target="closest div[data-name='meal-card']"
                                class="
                                    align-self-right
                                    bg-green-100
                                    hover:bg-green-200
                                    rounded
                                    p-1
                                    dark:text-black
                                ">
                                Duplicate
                            </button>
                            "#
                        )
                    };
                    format!(
                        r#"
                        {add_to_today_button}
                        <button
                            hx-delete="{delete_href}"
                            hx-target="closest div[data-name='meal-card']"
                            class="align-self-right bg-red-100 hover:bg-red-200
                            rounded p-1 dark:text-black"
                        >
                        Delete
                    </button>"#
                    )
                }
                None => "".into(),
            },
        };
        let background_style = if is_meal_before_today {
            "border-4 border-black dark:border-slate-200 md:dark:border-black"
        } else {
            r#"bg-gradient-to-br from-blue-100 via-sky-100 to-indigo-200
                dark:bg-gradient-to-br dark:from-blue-300 dark:via-cyan-300 =
                dark:to-indigo-300 dark:text-black"#
        };
        let warning = if self.show_ai_warning {
            r#"
                <p
                    class="text-xs bg-yellow-100 dark:bg-yellow-800
                    dark:text-slate-200 p-2 rounded-xl my-2"
                >
                    <span class="font-semibold">Warning:</span>
                    this is an AI estimate. Use discretion and re-prompt if it
                    doesn't look quite right!
                </p>
            "#
        } else {
            ""
        };
        format!(
            r##"
            <div
                class="rounded p-2 shadow sm:w-[20rem] mr-4
                {background_style}
                "
                data-name="meal-card"
                hx-swap="outerHTML"
            >
                <h1 class="text-2xl bold serif">{meal_name}</h1>
                <p class="text-lg"><b>Calories:</b> {calories} kcal</p>
                <p class="text-sm"><b>Protein:</b> {protein} grams</p>
                <p class="text-sm"><b>Carbs:</b> {carbs} grams</p>
                <p class="text-sm"><b>Fat:</b> {fat} grams</p>
                {warning}
                <div class="flex justify-end gap-2">
                    {actions}
                </div>
            </div>
            "##
        )
    }
}

struct MealSet<'a> {
    meals: &'a [Meal],
    user_timezone: Tz,
    next_page: i64,
    show_ai_warning: bool,
}
impl Component for MealSet<'_> {
    fn render(&self) -> String {
        let mut found_meal_before_today = false;
        let is_any_meal_during_today = self
            .meals
            .iter()
            .any(|m| !is_before_today(&m.info.created_at, self.user_timezone));
        let meals = self.meals.iter().enumerate().fold(
            String::new(),
            |mut acc, (i, meal)| {
                if !found_meal_before_today
                    && is_before_today(
                        &meal.info.created_at,
                        self.user_timezone,
                    )
                    && i != self.meals.len()
                    && is_any_meal_during_today
                {
                    found_meal_before_today = true;
                    acc.push_str(
                        // Note: the 20rem width matches the width of
                        // `MealCard`
                        r#"
                        <h2 class="
                            sticky
                            top-[-1px]
                            bg-zinc-50
                            dark:bg-slate-900
                            rounded
                            p-2
                            text-xl
                            font-bold
                        ">
                            Previous Food</h2>
                        <div class="w-[20rem] border-b-4 border-black">
                        <p class="text-xs my-4">
                            Items after this line were input yesterday or
                            before, and are not included in your daily totals
                            at the top.
                        </p>
                    </div>
                    "#,
                    )
                };
                acc.push_str(
                    &MealCard {
                        info: &meal.info,
                        meal_id: Some(meal.id),
                        actions: None,
                        rendering_behavior: RenderingBehavior::UseTimezone(
                            self.user_timezone,
                        ),
                        show_ai_warning: self.show_ai_warning,
                    }
                    .render(),
                );
                acc
            },
        );

        let page_usize: usize = MEAL_PAGE_SIZE.into();
        let next_page_div = if self.meals.len() == page_usize {
            let href = format!("{}?page={}", Route::ListMeals, self.next_page);
            format!(
                r#"
                <div hx-swap="outerHTML" hx-get="{href}" hx-trigger="revealed"></div>
                "#
            )
        } else {
            "".into()
        };
        format!(
            r#"
            {meals}
            {next_page_div}
            "#
        )
    }
}

pub struct CannotParse<'a> {
    pub llm_response: &'a str,
    pub original_user_prompt: &'a str,
    pub retry_route: Route,
}
impl Component for CannotParse<'_> {
    fn render(&self) -> String {
        let retry_route = &self.retry_route;
        let llm_response = clean(self.llm_response);
        let prompt = encode_quotes(&clean(self.original_user_prompt));
        format!(
            r##"
            <div class="prose max-w-[400px] dark:text-slate-200">
                <p><b>AI response:</b> {llm_response}</p>
                <form hx-post="{retry_route}" hx-target="#cal-chat-container">
                    <input type="hidden" value="{prompt}" name="meal_name" />
                    <button
                        class="
                            bg-red-100
                            dark:bg-red-800
                            dark:hover:bg-red-700
                            p-1
                            rounded
                            shadow
                            hover:bg-red-200
                    ">
                        Try Again
                    </button>
                </form>
            </div>
            "##
        )
    }
}

pub struct InputTooLong;
impl Component for InputTooLong {
    fn render(&self) -> String {
        let route = Route::HandleChat;
        format!(
            r##"
            <div class="prose">
                <p>Input is too long; please try again.</p>
                <button
                    hx-get="{route}"
                    hx-target="#cal-chat-container"
                    class="
                        bg-red-100
                        dark:bg-red-800
                        dark:hover:bg-red-700
                        p-1
                        rounded
                        shadow
                        hover:bg-red-200
                ">
                    Try Again
                </button>
            </div>
            "##
        )
    }
}

#[derive(Deserialize)]
pub struct ChatPayload {
    pub chat: String,
}

pub const SYSTEM_MSG: &str = "Act as an experienced nutritionist who can provide accurate calorie and macronutrient estimates. Your job is to give an approximation of calories and macros for foods that your client asks about. Do not hesitate to provide estimates for alcoholic beverages.

If the your client's query is extremely vague and you feel that you cannot provide an accurate calorie estimate, please ask for more details. In the case of severe ambiguity, give the most helpful response you can, and coach your client to give you a more descriptive query. If a client gives an ad-hoc measurement like, \"a handful,\" or \"a spoon full,\" or \"a small bowl,\" please always accept these measurements and convert them into exact measurements yourself. Keep in mind, if you hesitate to provide an estimate, your client may give up and avoid counting the calories of what they're eating. This can have serious adverse health effects, so only ask a follow-up question if your client has been extremely ambiguous, and it is impossible to provide a reasonable estimate.

Sometimes, users would like to manually add items. They may say, \"1200 calorie dinner.\" In this case, please just echo back, \"1200 calories, 0g of protein, 0g of fat, and 0g of carbohydrates.\" This is important to facilitate manually adding items. 

To complete this task, respond with calorie estimates and macronutrient estimates for the food I describe. A macronutrient breakdown is the amount of protein, carbohydrates, and fat, each measured in grams. Always provide exactly one number each for calories, grams of protein, grams of carbohydrates, and grams of fat so that your response is easy to parse. Please match this style in your response: \"The food you asked about has {} calories, {}g of protein, {}g of fat, and {}g of carbohydrates.";

pub async fn handle_chat(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(ChatPayload { chat }): Form<ChatPayload>,
) -> Result<impl IntoResponse, ServerError> {
    if chat.len() > config::CHAT_MAX_LEN {
        return Ok(InputTooLong {}.render());
    }
    let session = Session::from_headers_err(&headers, "handle chat")?;
    let preferences = session.get_preferences(&db).await?;
    let response = OpenAI::from_env()?
        .send_message(SYSTEM_MSG.into(), &chat)
        .await?;
    let Id { id } = query_as!(
        Id,
        "insert into openai_usage (prompt_tokens, completion_tokens, total_tokens)
        values ($1, $2, $3)
        returning id",
        response.usage.prompt_tokens,
        response.usage.completion_tokens,
        response.usage.total_tokens
    ).fetch_one(&db).await?;
    query!(
        "insert into openai_usage_user (usage_id, user_id) values ($1, $2)",
        id,
        session.user_id
    )
    .execute(&db)
    .await?;

    let parse_result = MealInfo::parse(&response.message, &chat);
    match parse_result {
        ParserResult::Ok(meal) => Ok(MealCard {
            info: &meal,
            meal_id: None,
            actions: Some(&NewMealOptions { info: &meal }),
            rendering_behavior: RenderingBehavior::UseTimezone(
                preferences.timezone,
            ),
            show_ai_warning: true,
        }
        .render()),
        ParserResult::FollowUp(_) => Ok(CannotParse {
            llm_response: &response.message,
            original_user_prompt: &chat,
            retry_route: Route::ChatForm,
        }
        .render()),
    }
}

#[derive(Deserialize)]
pub struct PrevPrompt {
    meal_name: String,
}

#[derive(Deserialize)]
pub struct Pagination {
    page: i64,
}

pub async fn chat_form(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    page: Option<Query<Pagination>>,
    prev_prompt: Option<Form<PrevPrompt>>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "chat form")?;
    let preferences = session.get_preferences(&db).await?;
    let page = page.map_or(0, |p| p.page);
    let meals = list_meals_op(&db, session.user_id, page).await?;
    let chat = Chat {
        meals: &meals,
        user_timezone: preferences.timezone,
        prompt: match prev_prompt {
            Some(ref form) => Some(&form.meal_name),
            None => None,
        },
        next_page: page + 1,
        post_request_handler: Route::HandleChat,
        is_anonymous: is_anon(&session.username),
    };
    let content = chat.render();
    Ok(content)
}

pub async fn list_meals_op<'a>(
    db: &PgPool,
    user_id: i32,
    page: i64,
) -> Aresult<Vec<Meal>> {
    let limit: i64 = MEAL_PAGE_SIZE.into();
    let offset = limit * page;

    struct Qres {
        id: i32,
        meal_name: String,
        calories: i32,
        fat_grams: i32,
        protein_grams: i32,
        carbohydrates_grams: i32,
        created_at: DateTime<Utc>,
    }
    let mut res = query_as!(
        Qres,
        "select
            id,
            name meal_name,
            calories,
            fat fat_grams,
            protein protein_grams,
            carbohydrates carbohydrates_grams,
            created_at
        from meal
        where user_id = $1
        order by created_at desc
        limit $2
        offset $3
        ",
        user_id,
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    Ok(res
        .drain(..)
        .map(|r| Meal {
            id: r.id,
            info: MealInfo {
                meal_name: r.meal_name,
                calories: r.calories,
                carbohydrates_grams: r.carbohydrates_grams,
                fat_grams: r.fat_grams,
                protein_grams: r.protein_grams,
                created_at: r.created_at,
            },
        })
        .collect::<Vec<Meal>>())
}

pub async fn handle_save_meal(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(meal): Form<MealInfo>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers(&headers)
        .ok_or_else(|| ServerError::forbidden("handle save meal"))?;
    let preferences = session.get_preferences(&db).await?;
    query!(
        "insert into meal
        (
            user_id,
            name,
            calories,
            fat,
            protein,
            carbohydrates,
            created_at
        )
        values ($1, $2, $3, $4, $5, $6, $7)",
        session.user_id,
        meal.meal_name,
        meal.calories,
        meal.fat_grams,
        meal.protein_grams,
        meal.carbohydrates_grams,
        meal.created_at
    )
    .execute(&db)
    .await?;
    let response_headers = client_events::reload_macros(HeaderMap::new());
    let meals = list_meals_op(&db, session.user_id, 0).await?;
    Ok((
        response_headers,
        Chat {
            meals: &meals,
            user_timezone: preferences.timezone,
            prompt: None,
            next_page: 1,
            post_request_handler: Route::HandleChat,
            is_anonymous: is_anon(&session.username),
        }
        .render(),
    ))
}

pub async fn list_meals(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Query(Pagination { page }): Query<Pagination>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "list meals")?;
    let (meals, preferences) = join![
        list_meals_op(&db, session.user_id, page),
        session.get_preferences(&db)
    ];
    let meals = meals?;
    let preferences = preferences?;

    Ok(MealSet {
        meals: &meals[..],
        next_page: page + 1,
        user_timezone: preferences.timezone,
        show_ai_warning: false,
    }
    .render())
}

struct PrevDayFormActions<'a> {
    info: &'a MealInfo,
}
impl Component for PrevDayFormActions<'_> {
    fn render(&self) -> String {
        let save_meal = Route::SaveMeal;
        let created_at = self.info.created_at.format("%d/%m/%Y");
        let script = include_str!("./custom_date_widget_helper.js");
        let meal_name = encode_quotes(&clean(&self.info.meal_name));
        let calories = self.info.calories;
        let protein = self.info.protein_grams;
        let carbs = self.info.carbohydrates_grams;
        let fat = self.info.fat_grams;
        format!(
            r##"
            <form
                hx-post="{save_meal}"
                hx-target="closest div[data-name='meal-card']"
                class="flex flex-col">
                <label for="created_date">
                    Date
                </label>
                <input
                    required
                    value="{created_at}"
                    type="date"
                    name="created_date"
                    id="created_date"
                />
                <!-- This field gets populated by JS when the buttons below are
                clicked -->
                <input type="hidden" name="created_at" id="created_at" />
                <input type="hidden" value="{meal_name}" name="meal_name" />
                <input type="hidden" value="{calories}" name="calories" />
                <input type="hidden" value="{protein}" name="protein_grams" />
                <input type="hidden" value="{carbs}" name="carbohydrates_grams" />
                <input type="hidden" value="{fat}" name="fat_grams" />
                <p class="text-sm">Approximately what time of day was this meal?</p>
                <button
                    class="block p-2 m-2 bg-blue-100 hover:bg-blue-200 rounded shadow hover:shadow-none"
                    id="breakfast"
                >
                    Breakfast
                </button>
                <button
                    class="block p-2 m-2 bg-blue-100 hover:bg-blue-200 rounded shadow hover:shadow-none"
                    id="lunch"
                >
                    Lunch
                </button>
                <button
                    class="block p-2 m-2 bg-blue-100 hover:bg-blue-200 rounded shadow hover:shadow-none"
                    id="dinner"
                >
                    Dinner
                </button>
                <button
                    class="block p-2 m-2 bg-blue-100 hover:bg-blue-200 rounded shadow hover:shadow-none"
                    id="evening"
                >
                    Evening
                </button>
                <script>(() => {{{script}}})();</script>
            </form>
            "##
        )
    }
}

pub async fn prev_day_meal_form(
    State(AppState { db }): State<AppState>,
    headers: HeaderMap,
    Form(meal): Form<MealInfo>,
) -> Result<impl IntoResponse, ServerError> {
    let session = Session::from_headers_err(&headers, "prev day meal form")?;
    let preferences = session.get_preferences(&db).await?;
    Ok(MealCard {
        info: &meal,
        meal_id: None,
        actions: Some(&PrevDayFormActions { info: &meal }),
        rendering_behavior: RenderingBehavior::UseTimezone(
            preferences.timezone,
        ),
        show_ai_warning: true,
    }
    .render())
}

#[cfg(test)]
mod test {
    use super::*;

    /// If an anon user has less than 3 meals, they won't get a warning.
    #[test]
    fn test_no_anon_warning_for_inactive_anon() {
        let mock_meals = &vec![
            Meal {
                id: 1,
                info: MealInfo {
                    calories: 1,
                    protein_grams: 1,
                    carbohydrates_grams: 1,
                    fat_grams: 1,
                    meal_name: "Snack".into(),
                    created_at: Utc::now(),
                },
            },
            Meal {
                id: 2,
                info: MealInfo {
                    calories: 1,
                    protein_grams: 1,
                    carbohydrates_grams: 1,
                    fat_grams: 1,
                    meal_name: "Snack".into(),
                    created_at: Utc::now(),
                },
            },
        ];
        let ui = Chat {
            meals: &mock_meals,
            prompt: None,
            user_timezone: Tz::UTC,
            next_page: 1,
            post_request_handler: Route::HandleChat,
            is_anonymous: true,
        }
        .render();
        assert!(!ui.contains("Anon Warning"));
    }

    /// Once an anon user has more than 3 meals, they'll see the anon warning.
    #[test]
    fn test_no_anon_warning_for_active_anon() {
        let mock_meals = &vec![
            Meal {
                id: 1,
                info: MealInfo {
                    calories: 1,
                    protein_grams: 1,
                    carbohydrates_grams: 1,
                    fat_grams: 1,
                    meal_name: "Snack".into(),
                    created_at: Utc::now(),
                },
            },
            Meal {
                id: 2,
                info: MealInfo {
                    calories: 1,
                    protein_grams: 1,
                    carbohydrates_grams: 1,
                    fat_grams: 1,
                    meal_name: "Snack".into(),
                    created_at: Utc::now(),
                },
            },
            Meal {
                id: 3,
                info: MealInfo {
                    calories: 1,
                    protein_grams: 1,
                    carbohydrates_grams: 1,
                    fat_grams: 1,
                    meal_name: "Snack".into(),
                    created_at: Utc::now(),
                },
            },
        ];
        let ui = Chat {
            meals: &mock_meals,
            prompt: None,
            user_timezone: Tz::UTC,
            next_page: 1,
            post_request_handler: Route::HandleChat,
            is_anonymous: true,
        }
        .render();
        assert!(ui.contains("Anon Warning"));
    }
}
