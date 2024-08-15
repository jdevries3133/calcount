delete from food where user_id not in (
    select id from users
);

alter table food
add constraint food_user_id_fkey
foreign key (user_id)
references users(id) on delete cascade;
