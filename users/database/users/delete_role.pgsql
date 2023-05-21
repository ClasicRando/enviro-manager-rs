create or replace procedure users.delete_role(
    name text
)
language sql
as $$
delete from users.roles r
where r.name = $1
$$;

revoke all on procedure users.delete_role from public;
grant execute on procedure users.delete_role to users_web;

comment on procedure users.delete_role IS $$
Remove a role. Will

Arguments:
name:
    Name of the existing role to delete
$$;
