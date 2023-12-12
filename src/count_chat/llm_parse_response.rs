//! An ad-hoc regex-y LLM response parser. Tries to tell the LLM to do better
//! next time an a resonably well-structured declarative way. Inspired by
//! https://www.youtube.com/watch?v=yj-wSRJwrrc.

use crate::components::Component;
use regex::{Captures, Regex};

#[derive(Debug)]
pub struct FollowUp {
    /// Response to send back to the LLM, hopefully to garner a better result.
    #[allow(dead_code)]
    llm_reply: String,
    /// Message that can be sent to the user, in case our retry policy has
    /// been exceeded.
    pub user_abort_msg: String,
}

#[derive(Debug)]
pub enum ParserResult<T> {
    /// If we were able to parse a structured type from the LLM response, here
    /// it is!
    Ok(T),
    /// The follow-up string is intended to be sent back to the LLM for a
    /// retry, though it should also be a
    FollowUp(FollowUp),
}

#[derive(Debug)]
pub struct MealInfo {
    calories: u16,
    protein_grams: u16,
    carbohydrates_grams: u16,
    fat_grams: u16,
}
impl Component for MealInfo {
    fn render(&self) -> String {
        let calories = self.calories;
        let protein = self.protein_grams;
        let carbs = self.carbohydrates_grams;
        let fat = self.fat_grams;
        format!(
            r#"
            <div class="bg-gradient-to-tr from-green-100 to-blue-100 rounded p-2 m-2 shadow">
                <p class="text-lg"><b>Calories:</b> {calories} kcal</p>
                <p><b>Protein:</b> {protein} grams</p>
                <p><b>Carbs:</b> {carbs} grams</p>
                <p><b>Fat:</b> {fat} grams</p>
            </div>
            "#
        )
    }
}
impl MealInfo {
    pub fn parse(llm_text: &str) -> ParserResult<Self> {
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
        (Ok(calories), Ok(protein_grams), Ok(fat_grams), Ok(carbohydrates_grams)) => {
            ParserResult::Ok(MealInfo {
                calories,
                protein_grams,
                carbohydrates_grams,
                fat_grams
            })
        },
        (calories, protein, fat, carbs) => {
            ParserResult::FollowUp(FollowUp {
                llm_reply: [calories, protein, fat, carbs].iter().fold(String::new(), |mut acc, res| {
                    if let Err(e) = res {
                        acc.push_str(e);
                    }
                    acc
                }),
                user_abort_msg: "Could not parse meal info from the LLM response. Try adding more specific details about your meal.".to_string()
            })
        }
    }
    }
}

/// Returns the parsed u16 inside the match object, or a response message for
/// the LLM.
fn handle_capture<'a>(
    mo: Option<&'a Captures>,
    describe_to_llm: &'a str,
) -> Result<u16, String> {
    match mo {
        Some(v) => {
            let start = v.get(1);
            let end = v.get(2);
            match (start, end) {
                (Some(s), Some(e)) => {
                    let lower_end = s.as_str().parse::<u16>();
                    let upper_end = e.as_str().parse::<u16>();
                    match (lower_end, upper_end) {
                        (Ok(l), Ok(u)) => {
                            Ok((l + u) / 2)
                        }
                        _ => Err(format!(
                            "I could not parse the range of {describe_to_llm} ({}-{}) as a number.\n",
                            s.as_str(),
                            e.as_str())
                        ),
                    }
                }
                (Some(s), None) => {
                    let value = s.as_str().parse::<u16>();
                    match value {
                        Ok(v) => Ok(v),
                        _ => Err(format!(
                            "I could not parse the string describing {describe_to_llm} ({}) as a number.\n",
                            s.as_str()))
                    }
                }
                _ => {
                    Err(format!("I could not find a count of {describe_to_llm} in that response.\n"))
                }
            }
        }
        None => Err(format!(
            "I could not find a count of {describe_to_llm} in that response.\n"
        )),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_meal_info() {
        let result = MealInfo::parse(
            "100-200 calories, 10g of fat, 11g of protein, 12g of carbs",
        );
        match result {
            ParserResult::Ok(meal) => {
                assert_eq!(meal.calories, 150);
                assert_eq!(meal.fat_grams, 10);
                assert_eq!(meal.protein_grams, 11);
                assert_eq!(meal.carbohydrates_grams, 12);
            }
            ParserResult::FollowUp(err) => {
                print!("{}", err.llm_reply);
                panic!("We should be able to parse this input");
            }
        }
    }

    #[test]
    fn test_other_filler_words() {
        let result = MealInfo::parse(
            "100-200 calories, 10g in fat, 11g in total protein, 12g of total carbs",
        );
        match result {
            ParserResult::Ok(meal) => {
                assert_eq!(meal.calories, 150);
                assert_eq!(meal.fat_grams, 10);
                assert_eq!(meal.protein_grams, 11);
                assert_eq!(meal.carbohydrates_grams, 12);
            }
            ParserResult::FollowUp(err) => {
                print!("{}", err.llm_reply);
                panic!("We should be able to parse this input");
            }
        }
    }

    #[test]
    fn test_missing_calories() {
        let result = MealInfo::parse(
            "100 calgories, 10g of fat, 11g of protein, 12g of carbs",
        );
        if let ParserResult::FollowUp(err) = result {
            assert_eq!(
                err.llm_reply,
                "I could not find a count of calories in that response.\n"
            );
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_unit() {
        let result = MealInfo::parse(
            "100 calories, 10 of fat, 11g of protein, 12g of carbs",
        );
        if let ParserResult::FollowUp(err) = result {
            assert_eq!(err.llm_reply, "I could not find a count of fat (in grams) in that response.\n");
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_fat() {
        let result =
            MealInfo::parse("100 calories, 11g of protein, 12g of carbs");
        if let ParserResult::FollowUp(err) = result {
            assert_eq!(err.llm_reply, "I could not find a count of fat (in grams) in that response.\n");
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_missing_two_properties() {
        let result = MealInfo::parse("100 calories, 12g of carbs");
        if let ParserResult::FollowUp(err) = result {
            assert_eq!(err.llm_reply, "I could not find a count of protein (in grams) in that response.\nI could not find a count of fat (in grams) in that response.\n");
        } else {
            panic!("expected an error");
        }
    }

    #[test]
    fn test_verbose_carbs() {
        let result = MealInfo::parse(
            "100 calories, 12g of fat, 13g of protein, 14g of carbohydrates",
        );
        if let ParserResult::Ok(res) = result {
            assert_eq!(res.carbohydrates_grams, 14);
        } else {
            panic!("expected an OK result");
        }
    }

    #[test]
    fn real_world_ex_1() {
        let result = MealInfo::parse(
            "Chex Mix usually contains around 120 calories, 2 grams of protein, 15 grams of carbohydrates, and 6 grams of fat per 1/2 cup serving.",
        );
        match result {
            ParserResult::Ok(meal) => {
                assert_eq!(meal.calories, 120);
                assert_eq!(meal.fat_grams, 6);
                assert_eq!(meal.protein_grams, 2);
                assert_eq!(meal.carbohydrates_grams, 15);
            }
            ParserResult::FollowUp(err) => {
                print!("{}", err.llm_reply);
                panic!("We should be able to parse this input");
            }
        }
    }
}
