alter table users add column created_at timestamp with time zone not null default now();
