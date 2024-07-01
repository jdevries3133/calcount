create sequence food_id_seq;
alter table food alter column id set default nextval('food_id_seq');
drop table meal;


