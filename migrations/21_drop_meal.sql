insert into food
select * from meal where id not in (
    select id from food
);
drop table meal;
