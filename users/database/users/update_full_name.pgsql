create or replace procedure users.update_full_name(
    uid uuid,
    new_first_name text,
    new_last_name text
)
language sql
as $$
update users.users u
set
    first_name = $2,
    last_name = $3
where u.uid = $1
$$;

grant execute on procedure users.update_full_name to users_web;

comment on procedure users.update_full_name IS $$
Update an existing user with new username provided

Arguments:
uid:
    UUID of the user to update
new_first_name:
    New first name to set for the specified user
new_last_name:
    New first name to set for the specified user
$$;
