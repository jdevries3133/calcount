-- Let's make a more generic `set_updated_at`, since we'll start reusing this
-- pattern for more `updated_at` fields.

create function set_updated_at()
returns trigger as $$
begin
    NEW.updated_on = now();
    return NEW;
end;
$$ language 'plpgsql';

create trigger post_set_updated_at_trigger_tmp
before update on post for each row
execute procedure set_updated_at();

drop trigger post_set_updated_at_trigger on post;
drop function post_set_updated_at;

alter trigger post_set_updated_at_trigger_tmp
on post
rename to post_set_updated_at_trigger;
