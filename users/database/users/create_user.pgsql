create or replace function users.create_user(
    first_name text,
    last_name text,
    username text,
    password text
)
returns uuid
language sql
as $$
insert into users.users as u (first_name,last_name,username,password)
values($1,$2,$3,crypt($4, gen_salt('bf')))
returning u.uid
$$;

grant execute on function users.create_user to users_web;

comment on function users.create_user IS $$
Create a new user with the provided details, returning the new user em_uid if successful.

Arguments:
first_name:
    First name of the new user
last_name:
    Last name of the new user
username:
    Username of the new user
password:
    Password of the new user, validated as the first step
$$;
