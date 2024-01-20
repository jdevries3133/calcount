#![allow(dead_code)]

use crate::{count_chat::Meal, prelude::*};
use chrono::Duration;

/// Balancing events typically encapsulate the previous goal, some effects,
/// and the new goal after those effects. Typically, these events will span
/// a period of one day, where the start and end times are going to be
/// adjusted to match the users' timezone.
///
/// This provides more detail than just providing an end goal, since it models
/// the sequene of events that led to the current goal.
#[derive(Debug)]
pub struct BalancingEvent<'a> {
    /// Start of the period that this event covers; typically the beginning of
    /// a user's day.
    start: DateTime<Utc>,
    /// End of the period; typically the end of a user's day.
    end: DateTime<Utc>,
    /// The goal that the user has actually set.
    user_input_calorie_goal: i32,
    previous_calorie_goal: i32,
    new_calorie_goal: i32,
    calories_consumed_during_period: i32,
    meals: &'a [Meal],
    user_tz: Tz,
}

impl Component for BalancingEvent<'_> {
    fn render(&self) -> String {
        let start = self.start.with_timezone(&self.user_tz).format("%b %d");
        let prev = self.previous_calorie_goal;
        let new = self.new_calorie_goal;
        let consumed = self.calories_consumed_during_period;
        let user_goal = self.user_input_calorie_goal;
        let meals = self.meals.iter().fold(String::new(), |mut acc, meal| {
            acc.push_str(&meal.render());
            acc
        });
        let meal_container = if meals.is_empty() {
            r#"
            <p class="text-xs italic">
                Zero meals have been eaten on this date
            </p>
            "#
            .into()
        } else {
            format!(
                r#"
                <details>
                    <summary>View Meals</summary>
                    <div class="flex gap-2 flex-wrap">
                    {meals}
                    </div>
                </details>
                "#
            )
        };
        let new_goal_description =
            if self.end > Utc::now().with_timezone(&self.user_tz) {
                "tommorow's goal"
            } else {
                "new goal"
            };
        format!(
            r#"
            <div class="bg-blue-100 dark:bg-blue-800 p-2 rounded m-2">
                <h2 class="text-lg">{start}</h2>
                <div class="font-mono">
                    <p>{prev} <sub>starting calculated goal</sub></p>
                    <p>- {consumed} <sub>calories consumed</sub>
                    <p>+ {user_goal} <sub>your goal</sub>
                    <p>-----------------------------------------</p>
                    <p>{new} <sub>{new_goal_description}</sub></p>
                </div>
                {meal_container}
            </div>
            "#
        )
    }
}

#[derive(Debug)]
pub struct BalancedCaloriesResult<'a> {
    /// The net calorie goal post-balancing
    pub current_calorie_goal: i32,
    pub details: Vec<BalancingEvent<'a>>,
}

/// `meals` must be provided sorted by date.
pub fn compute_balancing(
    now: DateTime<Utc>,
    user_timezone: Tz,
    calorie_goal: i32,
    meals: &[Meal],
) -> BalancedCaloriesResult<'_> {
    let mut details = vec![];
    let mut date = meals
        .first()
        .map_or(Utc::now(), |m| m.info.created_at)
        .with_timezone(&user_timezone)
        .with_hour(0)
        .expect("(1) zero is a valid hour, and we are not spanning a DST transition")
        .with_minute(0)
        .expect("(1) zero is a valid minute, and we are not spanning a DST transition")
        .with_second(0)
        .expect("(1) zero is a valid second, and we are not spanning a DST transition")
        .with_nanosecond(0)
        .expect("(1) zero is a valid nanosecond, and we are not spanning a DST transition");
    let mut meal_ptr = 0;
    while date < now {
        let mut calories_consumed = 0;
        let this_day_slice_start = meal_ptr;
        for meal in meals[meal_ptr..].iter() {
            let offset_from_date =
                meal.info.created_at.with_timezone(&user_timezone) - date;
            if offset_from_date > Duration::zero()
                && offset_from_date < Duration::days(1)
            {
                calories_consumed += meal.info.calories;
                meal_ptr += 1;
            }
        }
        let previous_calorie_goal = details
            .last()
            .map_or(calorie_goal, |e: &BalancingEvent| e.new_calorie_goal);

        details.push(BalancingEvent {
            start: date.with_timezone(&Utc),
            end: (date + Duration::days(1)).with_timezone(&Utc),
            user_input_calorie_goal: calorie_goal,
            previous_calorie_goal,
            new_calorie_goal: calorie_goal
                + details.last().map_or(calorie_goal, |e| e.new_calorie_goal)
                - calories_consumed,
            calories_consumed_during_period: calories_consumed,
            meals: &meals[this_day_slice_start..meal_ptr],
            user_tz: user_timezone,
        });
        date += Duration::days(1);
    }
    // Sort in descending order by date, so that the page reads from most recent
    // to oldest.
    details.sort_by(|a, b| {
        if a.start < b.start {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
    BalancedCaloriesResult {
        current_calorie_goal: details
            .iter()
            .find(|i| i.end < Utc::now().with_timezone(&user_timezone))
            .map_or(calorie_goal, |d| d.new_calorie_goal),
        details,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::count_chat::MealInfo;
    #[test]
    fn test_compute_balancing_subtracts_surplus_to_next_day() {
        let history = [Meal {
            id: 1,
            info: MealInfo {
                calories: 2100,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                meal_name: "test".into(),
                created_at: Utc::now()
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result = compute_balancing(Utc::now(), Tz::UTC, 2000, &history);
        assert_eq!(result.current_calorie_goal, 1900);
    }
    #[test]
    fn test_compute_balancing_subtracts_surplus_from_two_days() {
        let history = [
            Meal {
                id: 1,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "test".into(),
                    created_at: Utc::now()
                        - Duration::days(2)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            Meal {
                id: 2,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "test".into(),
                    created_at: Utc::now()
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result = compute_balancing(Utc::now(), Tz::UTC, 2000, &history);
        assert_eq!(result.current_calorie_goal, 1800);
    }
    #[test]
    fn test_compute_balancing_handles_one_day_gap() {
        let now = Utc::now();
        let history = [
            Meal {
                id: 1,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "3 days ago".into(),
                    created_at: now
                        - Duration::days(3)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            Meal {
                id: 2,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "yesterday".into(),
                    created_at: now
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result = compute_balancing(now, Tz::UTC, 2000, &history);
        assert_eq!(result.current_calorie_goal, 3800);
    }
    #[test]
    fn test_compute_balancing_adds_defecit_to_next_day() {
        let history = [Meal {
            id: 1,
            info: MealInfo {
                calories: 1900,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                meal_name: "test".into(),
                created_at: Utc::now()
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result = compute_balancing(Utc::now(), Tz::UTC, 2000, &history);
        assert_eq!(result.current_calorie_goal, 2100);
    }
    #[test]
    fn test_compute_balancing_creates_correct_event_log() {
        let now = Utc::now();
        let history = [
            Meal {
                id: 1,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "test".into(),
                    created_at: now
                        - Duration::days(3)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            Meal {
                id: 2,
                info: MealInfo {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    meal_name: "test".into(),
                    created_at: now
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result = compute_balancing(now, Tz::UTC, 2000, &history);
        // If we skip a day, we want an entry to exist for the skipped day,
        // which will show zero calories consumed.
        let skipped_day = result.details.iter().find(|e| {
            e.start.date_naive() == (now - Duration::days(2)).date_naive()
        });
        match skipped_day {
            Some(event) => {
                assert_eq!(event.calories_consumed_during_period, 0);
                assert_eq!(event.meals.len(), 0);
                assert_eq!(event.previous_calorie_goal, 1900);
                assert_eq!(event.new_calorie_goal, 3900);
            }
            None => panic!("skipped day does not have an event"),
        }
        assert_eq!(result.current_calorie_goal, 3800);
    }
}
