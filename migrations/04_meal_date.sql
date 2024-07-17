alter table meal add column created_at timestamp with time zone not null default now();
create index meal_time on meal (created_at);
