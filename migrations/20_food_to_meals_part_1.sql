create table if not exists food (like meal including all);

insert into food
select * from meal where id not in (
    select id from food
);
