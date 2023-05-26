create or replace procedure users.update_username(
    uid uuid,
    new_username text
)
security definer
language sql
as $$
update users.users u
set username = $2
where u.uid = $1
$$;

revoke all on procedure users.update_username from public;
grant execute on procedure users.update_username to users_web;

comment on procedure users.update_username IS $$
Update an existing user with new username provided. Will raise exception if the username already
exists.

Arguments:
uid:
    UUID of the user to update
new_username:
    New username to set for the specified user
$$;
