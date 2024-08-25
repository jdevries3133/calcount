use super::{
    food_card::{FoodCard, FoodIdentifiers, RenderingBehavior},
    FoodItem, FoodItemDetails,
};
use crate::{config::FOOD_PAGE_SIZE, prelude::*};

pub struct PreviousFood<'a> {
    pub meals: &'a Vec<FoodItem>,
    pub user_timezone: Tz,
    pub next_page: i64,
}
impl Component for PreviousFood<'_> {
    fn render(&self) -> String {
        let meals = FoodList {
            meals: &self.meals[..],
            user_timezone: self.user_timezone,
            next_page: self.next_page,
            show_ai_warning: false,
        }
        .render();
        let refresh_meals_href = format!("{}?page=0", Route::ListFood);
        format!(
            r#"
            <div
                class="flex flex-col gap-2 md:max-h-[70vh] md:overflow-y-auto"
            >
                <div
                    hx-get="{refresh_meals_href}"
                    hx-swap="innerHTML"
                    hx-trigger="reload-food from:body"
                    class="flex flex-col gap-2 mt-2 md:mt-0" >
                {meals}
                </div>
            </div>
            "#
        )
    }
}

pub struct FoodList<'a> {
    pub meals: &'a [FoodItem],
    pub user_timezone: Tz,
    pub next_page: i64,
    pub show_ai_warning: bool,
}
impl Component for FoodList<'_> {
    fn render(&self) -> String {
        let meals = self.meals.iter().fold(String::new(), |mut acc, meal| {
            acc.push_str(
                &FoodCard {
                    info: &meal.details,
                    identifiers: Some(FoodIdentifiers {
                        meal_id: meal.id,
                        eaten_event_id: meal.eaten_event_id,
                    }),
                    actions: None,
                    rendering_behavior: RenderingBehavior::UseTimezone(
                        self.user_timezone,
                    ),
                    show_ai_warning: self.show_ai_warning,
                }
                .render(),
            );
            acc
        });

        let page_usize: usize = FOOD_PAGE_SIZE.into();
        let next_page_div = if self.meals.len() == page_usize {
            let href = format!("{}?page={}", Route::ListFood, self.next_page);
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
    pub info: &'a FoodItemDetails,
}
impl Component for PrevDayFormActions<'_> {
    fn render(&self) -> String {
        let save_meal = Route::SaveFood;
        let default = Route::ChatForm;
        let eaten_at = self.info.eaten_at.format("%d/%m/%Y");
        let script = include_str!("./custom_date_widget_helper.js");
        let food_name = encode_quotes(&clean(&self.info.food_name));
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
                    value="{eaten_at}"
                    type="date"
                    name="created_date"
                    id="created_date"
                />
                <!-- This field gets populated by JS when the buttons below are
                clicked -->
                <input type="hidden" name="eaten_at" id="eaten_at" />
                <input type="hidden" value="{food_name}" name="food_name" />
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
                <button
                    class="block p-2 m-2 bg-red-100 hover:bg-red-200 rounded shadow hover:shadow-none"
                    hx-get="{default}"
                    hx-target="#cal-chat-container"
                >
                    Cancel
                </button>
                <script>(() => {{{script}}})();</script>
            </form>
            "##
        )
    }
}
