delete from food where user_id = 1;
-- 
insert into food (user_id, name, calories, carbohydrates, protein, fat, created_at)
values
    -- slight defecit
    (1, 'Cheese burger', 1000, 100, 100, 100, now() - interval '1 day'),
    (1, 'Ceasar salad', 900, 100, 100, 100, now() - interval '1 day'),

    -- teeter-totter day
    (1, 'French fries', 900, 100, 100, 100, now() - interval '2 days'),
    (1, 'Ribeye steak', 1100, 100, 100, 100, now() - interval '2 days'),

    -- We DO recover
    (1, 'Small portion of general tsos chicken', 500, 100, 100, 100, now() - interval '3 days'),
    (1, 'Large salad with ranch dressing', 1000, 100, 100, 100, now() - interval '3 days'),

    -- we don't recover
    (1, 'Phili cheesesteak', 1000, 100, 100, 100, now() - interval '4 days'),
    (1, 'Diner french toast', 1000, 100, 100, 100, now() - interval '4 days'),

    -- This becomes a 2500 kcal day
    (1, 'Cowboy omlette with home fries', 1000, 100, 100, 100, now() - interval '5 days'),
    (1, 'McDonalds double quarter pounder meal with a chocolate shake', 1500, 100, 100, 100, now() - interval '5 days')
;
select * from food;
