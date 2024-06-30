use super::{
    meal_card::{MealCard, RenderingBehavior},
    Meal, MealInfo,
};
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

pub struct PrevDayFormActions<'a> {
    pub info: &'a MealInfo,
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
