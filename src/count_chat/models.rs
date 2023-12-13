/// I am randomly switching over to `i32` for my numbers instead of `u16`
/// because it's occurred to me that getting a u16 in and out of the DB with
/// SQLx is going to be a PITA. I should change all my modeling around these
/// numbers to just use `i32` instead of `u16` at some point.
#[derive(Deserialize)]
pub struct Meal {
    name: String
    calories: i32,
    fat: i32,
    protein: i32,
    carbohydrates: i32
}
