//! An ad-hoc regex-y LLM response parser. Tries to tell the LLM to do better
//! next time an a resonably well-structured declarative way. Inspired by
//! https://www.youtube.com/watch?v=yj-wSRJwrrc.

use super::FoodItemDetails;
use crate::chrono_utils::utc_now;
use regex::{Captures, Regex};

impl FoodItemDetails {
    pub fn parse(llm_text: &str, food_name: &str) -> Result<Self, ()> {
        let calories_mo =
            Regex::new(r"(\d+)-?(\d+)? (of |in |total |the )*calories")
                .expect("cal regex is valid")
                .captures(llm_text);
        let protein_mo = Regex::new(
            r"(\d+)-?(\d+)?(g| grams) (of |in |total |the )*protein",
        )
        .expect("protein regex is valid")
        .captures(llm_text);
        let fat_mo =
            Regex::new(r"(\d+)-?(\d+)?(g| grams) (of |in |total |the )*fat")
                .expect("fat regex is valid")
                .captures(llm_text);
        let carbohydrates_mo = Regex::new(
            r"(\d+)-?(\d+)?(g| grams) (of |in |total |the )*(carbohydrates|carbs)",
        )
        .expect("carb regex is valid")
        .captures(llm_text);

        let calories = handle_capture(calories_mo.as_ref(), "calories");
        let protein = handle_capture(protein_mo.as_ref(), "protein (in grams)");
        let fat = handle_capture(fat_mo.as_ref(), "fat (in grams)");
        let carbohydrates = handle_capture(
            carbohydrates_mo.as_ref(),
            "carbohydrates (in grams)",
        );

        match (calories, protein, fat, carbohydrates) {
            (
                Ok(calories),
                Ok(protein_grams),
                Ok(fat_grams),
                Ok(carbohydrates_grams),
            ) => Ok(FoodItemDetails {
                food_name: food_name.to_string(),
                calories,
                protein_grams,
                carbohydrates_grams,
                fat_grams,
                eaten_at: utc_now(),
            }),
            _ => Err(()),
        }
    }
}

/// Returns the parsed i32 inside the match object, or a response message for
/// the LLM.
fn handle_capture<'a>(
    mo: Option<&'a Captures>,
    describe_to_llm: &'a str,
) -> Result<i32, String> {
    match mo {
        Some(v) => {
            let start = v.get(1);
            let end = v.get(2);
            match (start, end) {
                (Some(s), Some(e)) => {
                    let lower_end = s.as_str().parse::<i32>();
                    let upper_end = e.as_str().parse::<i32>();
                    match (lower_end, upper_end) {
                        (Ok(l), Ok(u)) => {
                            Ok((l + u) / 2)
                        }
                        _ => Err(format!(
                            "Could not parse the range of {describe_to_llm} ({}-{}) as a number.\n",
                            s.as_str(),
                            e.as_str())
                        ),
                    }
                }
                (Some(s), None) => {
                    let value = s.as_str().parse::<i32>();
                    match value {
                        Ok(v) => Ok(v),
                        _ => Err(format!(
                            "Could not parse the string describing {describe_to_llm} ({}) as a number.\n",
                            s.as_str()))
                    }
                }
                _ => {
                    Err(format!("Could not find a count of {describe_to_llm} in that response.\n"))
                }
            }
        }
        None => Err(format!(
            "Could not find a count of {describe_to_llm} in that response.\n"
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_food_info() {
        let result = FoodItemDetails::parse(
            "100-200 calories, 10g of fat, 11g of protein, 12g of carbs",
            "name",
        );
        match result {
            Ok(food) => {
                assert_eq!(food.calories, 150);
                assert_eq!(food.fat_grams, 10);
                assert_eq!(food.protein_grams, 11);
                assert_eq!(food.carbohydrates_grams, 12);
            }
            _ => {
                panic!("We should be able to parse this input");
            }
        }
    }

    #[test]
    fn test_other_filler_words() {
        let result = FoodItemDetails::parse(
            "100-200 calories, 10g in fat, 11g in total protein, 12g of total carbs",
            "name",
        );
        match result {
            Ok(food) => {
                assert_eq!(food.calories, 150);
                assert_eq!(food.fat_grams, 10);
                assert_eq!(food.protein_grams, 11);
                assert_eq!(food.carbohydrates_grams, 12);
            }
            _ => {
                panic!("We should be able to parse this input");
            }
        }
    }

    #[test]
    fn test_missing_calories() {
        let result = FoodItemDetails::parse(
            "100 calgories, 10g of fat, 11g of protein, 12g of carbs",
            "name",
        );
        if result.is_ok() {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_unit() {
        let result = FoodItemDetails::parse(
            "100 calories, 10 of fat, 11g of protein, 12g of carbs",
            "name",
        );
        if result.is_ok() {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_fat() {
        let result = FoodItemDetails::parse(
            "100 calories, 11g of protein, 12g of carbs",
            "name",
        );
        if result.is_ok() {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_two_properties() {
        let result =
            FoodItemDetails::parse("100 calories, 12g of carbs", "name");
        if result.is_ok() {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_verbose_carbs() {
        let result = FoodItemDetails::parse(
            "100 calories, 12g of fat, 13g of protein, 14g of carbohydrates",
            "name",
        );
        if let Ok(res) = result {
            assert_eq!(res.carbohydrates_grams, 14);
        } else {
            panic!("expected an OK result");
        }
    }

    #[test]
    fn real_world_ex_1() {
        let result = FoodItemDetails::parse(
            "Chex Mix usually contains around 120 calories, 2 grams of protein, 15 grams of carbohydrates, and 6 grams of fat per 1/2 cup serving.",
            "name",
        );
        match result {
            Ok(food) => {
                assert_eq!(food.calories, 120);
                assert_eq!(food.fat_grams, 6);
                assert_eq!(food.protein_grams, 2);
                assert_eq!(food.carbohydrates_grams, 15);
            }
            _ => {
                panic!("We should be able to parse this input");
            }
        }
    }
}
