create table meal(
    id serial primary key,
    user_id int not null references users(id),
    name text not null,
    calories int not null,
    carbohydrates int not null,
    protein int not null,
    fat int not null
);
