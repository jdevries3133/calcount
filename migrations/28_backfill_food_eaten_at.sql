insert into food_eaten_event (food_id, user_id, eaten_at)
select id, user_id, eaten_at
from food;
