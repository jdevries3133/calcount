alter table user_preference drop constraint user_preference_user_id_fkey;

alter table user_preference add constraint user_preference_user_id_fkey
foreign key (user_id) references users(id) on delete cascade;
