create table if not exists food_eaten_event(
    id serial primary key not null,
    food_id int not null references food(id),
    user_id int not null references users(id),
    eaten_at timestamp with time zone not null default now()
);
create index if not exists idx_eaten_at on food_eaten_event (eaten_at);

insert into food_eaten_event (food_id, user_id, eaten_at)
select id, user_id, eaten_at
from food;
