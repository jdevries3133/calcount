create table audit_stripe_webhooks(
    id serial primary key not null,
    payload text not null,
    created_at timestamp with time zone not null default now(),
    includes_usable_update boolean not null
);

create table subscription_type(
  id serial primary key not null,
  name text not null,
  monthly_recurring_revenue_cents int not null
);

insert into subscription_type (name, monthly_recurring_revenue_cents) values
    ('initializing', 0),
    ('basic plan', 500),
    ('free', 0),
    ('unsubscribed', 0),
    ('free trial', 0)
;

alter table users add column subscription_type_id int not null references subscription_type(id) default 3;

-- Then, drop the default once we've populated the existing user, since we
-- don't really want a default value which depends on neighboring rows after
-- the initial migration.
alter table users alter column subscription_type_id drop default;
