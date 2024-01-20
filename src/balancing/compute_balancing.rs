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
struct BalancingEvent<'a> {
    /// Start of the period that this event covers; typically the beginning of
    /// a user's day.
    start: DateTime<Utc>,
    /// End of the period; typically the end of a user's day.
    end: DateTime<Utc>,
    previous_calorie_goal: i32,
    new_calorie_goal: i32,
    calories_consumed_during_period: i32,
    meals: &'a [Meal],
}

#[derive(Debug)]
pub struct BalancedCaloriesResult<'a> {
    /// The net calorie goal post-balancing
    pub current_calorie_goal: i32,
    details: Vec<BalancingEvent<'a>>,
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
    while date < now - Duration::days(1) {
        let mut calories_consumed = 0;
        let this_day_slice_start = meal_ptr;
        let mut this_day_slice_end = this_day_slice_start;
        for (i, meal) in meals[meal_ptr..].iter().enumerate() {
            if meal.info.created_at.date_naive() == date.date_naive() {
                calories_consumed += meal.info.calories;
                this_day_slice_end = i + meal_ptr;
            } else {
                meal_ptr += i;
                break;
            }
        }
        let previous_calorie_goal = details
            .last()
            .map_or(calorie_goal, |e: &BalancingEvent| e.new_calorie_goal);

        details.push(BalancingEvent {
            start: date.with_timezone(&Utc),
            end: (date + Duration::days(1)).with_timezone(&Utc),
            previous_calorie_goal,
            new_calorie_goal: calorie_goal
                + details.last().map_or(calorie_goal, |e| e.new_calorie_goal)
                - calories_consumed,
            calories_consumed_during_period: calories_consumed,
            meals: &meals[this_day_slice_start..this_day_slice_end],
        });
        date += Duration::days(1);
    }
    BalancedCaloriesResult {
        current_calorie_goal: details
            .last()
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
