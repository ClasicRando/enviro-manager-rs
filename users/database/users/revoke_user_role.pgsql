create or replace procedure users.revoke_user_role(
    uid uuid,
    role text
)
language sql
as $$
delete from users.user_roles ur
where
    ur.uid = $1
    and ur.role = $2;
$$;

revoke all on procedure users.revoke_user_role from public;
grant execute on procedure users.revoke_user_role to users_web;

comment on procedure users.revoke_user_role IS $$
Revoke a role for a specified user.

Arguments:
uid:
    UUID specifying the user to revoke a role
role:
    Name of the role to revoke from the specified user
$$;
