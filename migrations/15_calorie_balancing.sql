create table balancing_checkpoint(
    user_id int not null references users(id),
    ignore_before date not null,
    primary key (user_id, ignore_before)
);
