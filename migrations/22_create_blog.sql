create table post(
    id serial primary key not null,
    title text not null,
    summary text not null,
    post_markdown text not null,
    created_at timestamp with time zone not null default now(),
    updated_at timestamp with time zone not null
);

create function post_set_updated_at()
returns trigger as $$
begin
    NEW.updated_on = now();
    return NEW;
end;
$$ language 'plpgsql';

create trigger post_set_updated_at_trigger
before update on post for each row
execute procedure post_set_updated_at();

create table comment(
    id serial primary key not null,
    user_id int not null references users(id),
    post_id int not null references post(id),
    created_at timestamp with time zone not null default now(),
    body text not null
);
