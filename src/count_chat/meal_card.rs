use crate::{chrono_utils::is_before_today, components::Void, prelude::*};

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
        let datetime = match self.rendering_behavior {
            RenderingBehavior::UseTimezone(tz) => {
                self.info.created_at.with_timezone(&tz)
            }
            RenderingBehavior::RenderAsToday => {
                Utc::now().with_timezone(&Tz::UTC)
            }
        };
        let date_str = datetime.format("%b %e");
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
        let date_style = if is_meal_before_today {
            "text-right col-span-2 text-sm text-slate-700 dark:text-white"
        } else {
            "text-right col-span-2 text-sm text-slate-700"
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
                <div class="grid grid-cols-7">
                    <h1 class="col-span-5 text-2xl bold serif">{meal_name}</h1>
                    <span class="{date_style}">{date_str}</span>
                </div>
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
