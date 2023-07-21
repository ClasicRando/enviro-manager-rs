create table if not exists users.users (
    uid uuid primary key default gen_random_uuid (),
    full_name text not null check(data_check.check_not_blank_or_empty(full_name)),
    username text not null check(data_check.check_not_blank_or_empty(username)) unique,
    password text not null check(data_check.check_not_blank_or_empty(password))
);

call audit.audit_table('users.users');

comment on table users.users is
'All users that will access EnviroManager resources';
comment on column users.users.uid is
'Unique identifier for each user';
comment on column users.users.full_name is
'Full name of the user for message/display purposes';
comment on column users.users.username is
'Unique string value to signify the user. Used for login purposes';
comment on column users.users.password is
'Hashed and salted password for the user. Used for login purposes';
