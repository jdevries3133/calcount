alter table user_preference add column
calorie_balancing_enabled boolean not null default false;

alter table user_preference add column
calorie_balancing_min_calories int;

alter table user_preference add column
calorie_balancing_max_calories int;
