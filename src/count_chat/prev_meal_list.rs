use super::{Meal, MealInfo};
use crate::{
    chrono_utils::is_before_today, config::MEAL_PAGE_SIZE, prelude::*,
};

pub struct PreviousMeals<'a> {
    pub meals: &'a Vec<Meal>,
    pub user_timezone: Tz,
    pub next_page: i64,
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
                    dark:bg-blue-950
                    rounded
                    p-2
                    mt-2
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
                    dark:bg-blue-950
                    rounded
                    p-2
                    mt-2
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
                                    bg-emerald-100
                                    hover:bg-emerald-200
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
                                    bg-emerald-100
                                    hover:bg-emerald-200
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
                class="rounded p-2 shadow sm:w-[20rem]
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

pub struct MealSet<'a> {
    pub meals: &'a [Meal],
    pub user_timezone: Tz,
    pub next_page: i64,
    pub show_ai_warning: bool,
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
