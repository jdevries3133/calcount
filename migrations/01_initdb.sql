create extension if not exists "uuid-ossp";

create table users(
    id serial primary key,
    username varchar(255) unique not null,
    email varchar(255) unique not null,
    salt varchar(255) not null,
    digest varchar(255) not null
);

create table item(
    id serial primary key,
    name varchar(255) not null
);
