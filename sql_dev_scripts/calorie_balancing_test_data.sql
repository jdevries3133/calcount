delete from food where user_id = 1;
-- 
insert into food (user_id, name, calories, carbohydrates, protein, fat, eaten_at)
values
    -- slight defecit
    (1, 'slight-defecit', 1000, 100, 100, 100, now() - interval '1 day'),
    (1, 'slight-defecit', 900, 100, 100, 100, now() - interval '1 day'),

    -- teeter-totter day
    (1, 'totter', 900, 100, 100, 100, now() - interval '2 days'),
    (1, 'teeter', 1100, 100, 100, 100, now() - interval '2 days'),

    -- We DO recover
    (1, 'recover', 500, 100, 100, 100, now() - interval '3 days'),
    (1, 'recover', 1000, 100, 100, 100, now() - interval '3 days'),

    -- we don't recover
    (1, 'no-recover', 1000, 100, 100, 100, now() - interval '4 days'),
    (1, 'no-recover', 1000, 100, 100, 100, now() - interval '4 days'),

    -- This becomes a 2500 kcal day
    (1, 'excess', 1000, 100, 100, 100, now() - interval '5 days'),
    (1, 'excess', 1500, 100, 100, 100, now() - interval '5 days')
;
select * from food;
