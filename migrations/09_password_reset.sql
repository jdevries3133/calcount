create table password_reset_link(
    user_id int not null primary key references users(id),
    created_at timestamp with time zone not null default now(),
    slug text not null
);
