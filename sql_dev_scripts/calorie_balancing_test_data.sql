delete from food_eaten_event where user_id = 1;
delete from food where user_id = 1;

-- slight defecit
with inserted_slight_deficit as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'slight-defecit', 1000, 100, 100, 100),
        (1, 'slight-defecit', 900, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '1 day'
from inserted_slight_deficit;

-- teeter-totter day
with inserted_teeter_totter as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'totter', 900, 100, 100, 100),
        (1, 'teeter', 1100, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '2 days'
from inserted_teeter_totter;

-- We DO recover
with inserted_recover as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'recover', 500, 100, 100, 100),
        (1, 'recover', 1000, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '3 day'
from inserted_recover;

with inserted_no_recover as (
    -- we don't recover
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'no-recover', 1000, 100, 100, 100),
        (1, 'no-recover', 1000, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '4 day'
from inserted_no_recover;

with inserted_excess as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
    -- This becomes a 2500 kcal day
        (1, 'excess', 1000, 100, 100, 100),
        (1, 'excess', 1500, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '5 day'
from inserted_excess;

select * from food where user_id = 1;
