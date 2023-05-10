create table if not exists users.users (
    em_uid bigint primary key generated always as identity,
    first_name text not null check(data_check.check_not_blank_or_empty(first_name)),
    last_name text not null check(data_check.check_not_blank_or_empty(last_name)),
    username text not null check(data_check.check_not_blank_or_empty(last_name)) unique,
    password text not null check(data_check.check_not_blank_or_empty(last_name))
);

call audit.audit_table('users.users');

revoke all on users.users from public;

comment on table users.users is
'All users that will access EnviroManager resources';
comment on column users.users.em_uid is
'Unique identifier for each user';
comment on column users.users.first_name is
'First name of the user for message/display purposes';
comment on column users.users.last_name is
'Last name of the user for message/display purposes';
comment on column users.users.username is
'Unique string value to signify the user. Used for login purposes';
comment on column users.users.password is
'Hashed and salted password for the user. Used for login purposes';
