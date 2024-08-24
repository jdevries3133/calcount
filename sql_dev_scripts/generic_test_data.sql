delete from food_eaten_event where user_id = 1;
delete from food where user_id = 1;

-- insert slight deficit foods and their corresponding events
with inserted_slight_deficit as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'cheese burger', 1000, 100, 100, 100),
        (1, 'ceasar salad', 900, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '1 day'
from inserted_slight_deficit;

-- insert teeter-totter day foods and their corresponding events
with inserted_teeter_totter as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'french fries', 900, 100, 100, 100),
        (1, 'ribeye steak', 1100, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '2 days'
from inserted_teeter_totter;

-- insert recover day foods and their corresponding events
with inserted_recover as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'small portion of general tsos chicken', 500, 100, 100, 100),
        (1, 'large salad with ranch dressing', 1000, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '3 days'
from inserted_recover;

-- insert no-recover day foods and their corresponding events
with inserted_no_recover as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'phili cheesesteak', 1000, 100, 100, 100),
        (1, 'diner french toast', 1000, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '4 days'
from inserted_no_recover;

-- insert excess day foods and their corresponding events
with inserted_excess as (
    insert into food (user_id, name, calories, carbohydrates, protein, fat)
    values
        (1, 'cowboy omlette with home fries', 1000, 100, 100, 100),
        (1, 'mcdonalds double quarter pounder meal with a chocolate shake', 1500, 100, 100, 100)
    returning id
)
insert into food_eaten_event (user_id, food_id, eaten_at)
select 1, id, now() - interval '5 days'
from inserted_excess;

-- select all data from the food table
select * from food;
