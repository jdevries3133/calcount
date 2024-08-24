alter table food add column eaten_at timestamp with time zone;

insert into food (id, eaten_at)
select food_id, eaten_at from food_eaten_event
on conflict (id) do update set eaten_at = EXCLUDED.eaten_at;

alter table food alter column eaten_at set not null;
