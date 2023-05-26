create or replace procedure users.reset_password(
    uid uuid,
    new_password text
)
security definer
language sql
as $$
update users.users u
set password = crypt($2, gen_salt('bf'))
where u.uid = $1
$$;

revoke all on procedure users.reset_password from public;
grant execute on procedure users.reset_password to users_web;

comment on procedure users.reset_password IS $$
Update an existing user with new username provided. Will raise exception if the username already
exists.

Arguments:
username:
    Unique name of the user to update
password:
    Current password of the user to verify that the update to username is okay
new_password:
    New password to set for the specified user
$$;
