create or replace procedure users.update_full_name(
    uid uuid,
    new_full_name text
)
security definer
language sql
as $$
update users.users u
set full_name = $2
where u.uid = $1
$$;

revoke all on procedure users.update_full_name from public;
grant execute on procedure users.update_full_name to users_web;

comment on procedure users.update_full_name IS $$
Update an existing user with new username provided

Arguments:
uid:
    UUID of the user to update
new_full_name:
    New full name to set for the specified user
$$;
