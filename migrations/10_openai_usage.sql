create table openai_usage(
    id serial primary key not null,
    prompt_tokens int not null,
    completion_tokens int not null,
    total_tokens int not null
);

create table openai_usage_user(
    usage_id int not null references openai_usage(id),
    user_id int not null references users(id),
    primary key (usage_id, user_id)
);
