#![allow(dead_code)]

use crate::{count_chat::FoodItem, prelude::*};
use chrono::Duration;
use std::cmp::{max, min};

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
    previous_excess_calories: i32,
    new_calorie_goal: i32,
    calories_consumed_during_period: i32,
    /// With calorie balancing, calories in excess of the user's minimum or
    /// maximum limit will go into this bucket.
    calories_to_be_applied_at_a_later_date: i32,
    food_items: &'a [FoodItem],
    user_tz: Tz,
}

impl Component for BalancingEvent<'_> {
    fn render(&self) -> String {
        let start = self.start.with_timezone(&self.user_tz).format("%b %d");
        let prev = self.previous_calorie_goal;
        let new = self.new_calorie_goal;
        let consumed = self.calories_consumed_during_period;
        let user_goal = self.user_input_calorie_goal;
        let food_items =
            self.food_items.iter().fold(String::new(), |mut acc, food| {
                acc.push_str(&food.render());
                acc
            });
        let food_container = if food_items.is_empty() {
            r#"
            <p class="text-xs italic">
                No food was eaten on this date
            </p>
            "#
            .into()
        } else {
            format!(
                r#"
                <details>
                    <summary>View Food</summary>
                    <div class="flex gap-2 flex-wrap">
                    {food_items}
                    </div>
                </details>
                "#
            )
        };
        let new_goal_description =
            if self.end > utc_now().with_timezone(&self.user_tz) {
                "tommorow's goal"
            } else {
                "new goal"
            };
        let extra = self.calories_to_be_applied_at_a_later_date;
        let naive_goal =
            self.new_calorie_goal + self.calories_to_be_applied_at_a_later_date;
        let abs_prev_extra = self.previous_excess_calories.abs();
        let prev_extra = if self.previous_excess_calories == 0 {
            "".to_string()
        } else if self.previous_calorie_goal.is_positive() {
            format!("<p>+ {abs_prev_extra} <sub>rollover from previous days</sub></p>")
        } else {
            format!("<p>- {abs_prev_extra} <sub>rollover from previous days</sub></p>")
        };
        format!(
            r#"
            <div class="bg-blue-200 dark:bg-blue-950 p-2 rounded m-2">
                <h2 class="text-lg">{start}</h2>
                <div class="font-mono">
                    <p>{prev} <sub>starting calculated goal</sub></p>
                    {prev_extra}
                    <p>- {consumed} <sub>calories consumed</sub>
                    <p>+ {user_goal} <sub>your goal</sub>
                    <hr class="my-2" />
                    <p>{naive_goal} <sub>total</sub></p>
                    <p>=> {new} <sub>{new_goal_description}</sub></p>
                    <p>=> {extra} <sub>calories exceeding limits</sub></p>
                </div>
                {food_container}
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

/// `food` must be provided sorted by date.
pub fn compute_balancing(
    now: DateTime<Utc>,
    user_timezone: Tz,
    calorie_goal: i32,
    max_calories: Option<i32>,
    min_calories: Option<i32>,
    food_items: &[FoodItem],
) -> BalancedCaloriesResult<'_> {
    let max_calories = max_calories.unwrap_or(i32::MAX);
    let min_calories = min_calories.unwrap_or(0);
    // Hm, I wonder if max / min calories should be stored as an offset from
    // the goal to avoid this invariant condition.
    if min_calories > calorie_goal {
        panic!("min calories cannot be greater than the calorie goal");
    };
    if max_calories < calorie_goal {
        panic!("max calories cannot be less than the calorie goal");
    };
    let mut details = vec![];
    let mut date = food_items
        .first()
        .map_or(utc_now(), |m| m.details.eaten_at)
        .with_timezone(&user_timezone)
        .with_hour(0)
        .expect("(1) zero is a valid hour, and we are not spanning a DST transition")
        .with_minute(0)
        .expect("(1) zero is a valid minute, and we are not spanning a DST transition")
        .with_second(0)
        .expect("(1) zero is a valid second, and we are not spanning a DST transition")
        .with_nanosecond(0)
        .expect("(1) zero is a valid nanosecond, and we are not spanning a DST transition");
    let mut food_ptr = 0;
    while date < now {
        let mut calories_consumed = 0;
        let this_day_slice_start = food_ptr;
        for food in food_items[food_ptr..].iter() {
            let offset_from_date =
                food.details.eaten_at.with_timezone(&user_timezone) - date;
            if offset_from_date > Duration::zero()
                && offset_from_date < Duration::days(1)
            {
                calories_consumed += food.details.calories;
                food_ptr += 1;
            }
        }

        let previous_calorie_goal = details
            .last()
            .map_or(calorie_goal, |e: &BalancingEvent| e.new_calorie_goal);
        let previous_remainder = details
            .last()
            .map_or(0, |e| e.calories_to_be_applied_at_a_later_date);

        let goal_before_applying_limits = calorie_goal
            + details.last().map_or(calorie_goal, |e| e.new_calorie_goal)
            - calories_consumed;
        let goal_with_remainder =
            goal_before_applying_limits + previous_remainder;
        let limited_goal =
            min(max_calories, max(min_calories, goal_with_remainder));
        let new_remainder = goal_with_remainder - limited_goal;

        details.push(BalancingEvent {
            start: date.with_timezone(&Utc),
            end: (date + Duration::days(1)).with_timezone(&Utc),
            user_input_calorie_goal: calorie_goal,
            previous_calorie_goal,
            previous_excess_calories: previous_remainder,
            new_calorie_goal: limited_goal,
            calories_consumed_during_period: calories_consumed,
            calories_to_be_applied_at_a_later_date: new_remainder,
            food_items: &food_items[this_day_slice_start..food_ptr],
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
            .find(|i| i.end < utc_now().with_timezone(&user_timezone))
            .map_or(calorie_goal, |d| d.new_calorie_goal),
        details,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::count_chat::FoodItemDetails;
    #[test]
    fn test_compute_balancing_subtracts_surplus_to_next_day() {
        let history = [FoodItem {
            id: 1,
            details: FoodItemDetails {
                calories: 2100,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                food_name: "test".into(),
                eaten_at: utc_now()
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result =
            compute_balancing(utc_now(), Tz::UTC, 2000, None, None, &history);
        assert_eq!(result.current_calorie_goal, 1900);
    }
    #[test]
    fn test_compute_balancing_subtracts_surplus_from_two_days() {
        let history = [
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: utc_now()
                        - Duration::days(2)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 2,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: utc_now()
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result =
            compute_balancing(utc_now(), Tz::UTC, 2000, None, None, &history);
        assert_eq!(result.current_calorie_goal, 1800);
    }
    #[test]
    fn test_compute_balancing_handles_one_day_gap() {
        let now = utc_now();
        let history = [
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "3 days ago".into(),
                    eaten_at: now
                        - Duration::days(3)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 2,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "yesterday".into(),
                    eaten_at: now
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result =
            compute_balancing(now, Tz::UTC, 2000, None, None, &history);
        assert_eq!(result.current_calorie_goal, 3800);
    }
    #[test]
    fn test_compute_balancing_adds_defecit_to_next_day() {
        let history = [FoodItem {
            id: 1,
            details: FoodItemDetails {
                calories: 1900,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                food_name: "test".into(),
                eaten_at: utc_now()
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result =
            compute_balancing(utc_now(), Tz::UTC, 2000, None, None, &history);
        assert_eq!(result.current_calorie_goal, 2100);
    }
    #[test]
    fn test_compute_balancing_creates_correct_event_log() {
        let now = utc_now();
        let history = [
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(3)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 2,
                details: FoodItemDetails {
                    calories: 2100,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result =
            compute_balancing(now, Tz::UTC, 2000, None, None, &history);
        // If we skip a day, we want an entry to exist for the skipped day,
        // which will show zero calories consumed.
        let skipped_day = result.details.iter().find(|e| {
            e.start.date_naive() == (now - Duration::days(2)).date_naive()
        });
        match skipped_day {
            Some(event) => {
                assert_eq!(event.calories_consumed_during_period, 0);
                assert_eq!(event.food_items.len(), 0);
                assert_eq!(event.previous_calorie_goal, 1900);
                assert_eq!(event.new_calorie_goal, 3900);
            }
            None => panic!("skipped day does not have an event"),
        }
        assert_eq!(result.current_calorie_goal, 3800);
    }
    #[test]
    fn test_max_limit() {
        let now = utc_now();
        let history = [FoodItem {
            id: 1,
            details: FoodItemDetails {
                calories: 1000,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                food_name: "test".into(),
                eaten_at: now
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result =
            compute_balancing(now, Tz::UTC, 2000, Some(2200), None, &history);
        assert_eq!(result.current_calorie_goal, 2200);
        assert_eq!(
            result
                .details
                .last()
                .unwrap()
                .calories_to_be_applied_at_a_later_date,
            800
        );
    }
    #[test]
    fn test_min_limit() {
        let now = utc_now();
        let history = [FoodItem {
            id: 1,
            details: FoodItemDetails {
                calories: 3000,
                fat_grams: 0,
                protein_grams: 0,
                carbohydrates_grams: 0,
                food_name: "test".into(),
                eaten_at: now
                    - Duration::days(1)
                        .to_std()
                        .expect("can convert days to std"),
            },
        }];
        let result =
            compute_balancing(now, Tz::UTC, 2000, None, Some(1800), &history);
        assert_eq!(result.current_calorie_goal, 1800);
        assert_eq!(
            result
                .details
                .last()
                .unwrap()
                .calories_to_be_applied_at_a_later_date,
            -800
        );
    }
    #[test]
    fn test_min_and_max_limits_practical_example() {
        let now = utc_now();
        let history = [
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 4000,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(6)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 2000,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(4)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 2400,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(3)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 1800,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(2)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
            FoodItem {
                id: 1,
                details: FoodItemDetails {
                    calories: 1000,
                    fat_grams: 0,
                    protein_grams: 0,
                    carbohydrates_grams: 0,
                    food_name: "test".into(),
                    eaten_at: now
                        - Duration::days(1)
                            .to_std()
                            .expect("can convert days to std"),
                },
            },
        ];
        let result = compute_balancing(
            now,
            Tz::UTC,
            2000,
            Some(2200),
            Some(1800),
            &history,
        );
        // This is the day that we eat 2400 calories. Since our goal is 2000,
        // and the limit is 1800, we end up with a next-day goal of 1800,
        // and -200 in the "apply to a future date," category.
        let overeating_day = &result.details[3];
        assert_eq!(overeating_day.new_calorie_goal, 1800);
        assert_eq!(overeating_day.calories_to_be_applied_at_a_later_date, -200);

        // This is the day where we eat 1000 calories; after starting the day
        // with an 1800 calorie goal.
        let undereating_day = &result.details[1];
        assert_eq!(undereating_day.new_calorie_goal, 2200);
        assert_eq!(undereating_day.calories_to_be_applied_at_a_later_date, 600);

        assert_eq!(result.current_calorie_goal, 2200);
    }
}
