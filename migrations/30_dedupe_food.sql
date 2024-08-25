begin;

with dupes as (
    select
        id,
        user_id,
        name,
        calories,
        carbohydrates,
        protein,
        fat,
        min(id) over (partition by user_id, name, calories, carbohydrates, protein, fat) as keep_id
    from food
)
update food_eaten_event 
set food_id = dupes.keep_id
from dupes
where food_eaten_event.food_id = dupes.id and dupes.id != dupes.keep_id;

with dupes as (
    select
        id,
        user_id,
        name,
        calories,
        carbohydrates,
        protein,
        fat,
        row_number() over (partition by user_id, name, calories, carbohydrates, protein, fat order by id) as rn
    from food
)
delete from food 
where id in (
    select id 
    from dupes 
    where rn > 1
);

commit;
