create table user_preference(
    user_id int not null primary key references users(id),
    timezone text not null
);
