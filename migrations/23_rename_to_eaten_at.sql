alter table food add column
eaten_at timestamp with time zone
default now()
not null;

create index eaten_time on food (eaten_at);

update food set eaten_at = created_at;
