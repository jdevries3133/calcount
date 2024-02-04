alter table openai_usage add column created_at timestamp with time zone default now();
