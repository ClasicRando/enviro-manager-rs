create or replace procedure users.add_user_role(
    uid uuid,
    role text
)
language sql
as $$
insert into users.user_roles(em_uid, role)
select u.em_uid, $2 role
from users.users u
where u.uid = $1
on conflict (em_uid, role) do nothing;
$$;

revoke all on procedure users.add_user_role from public;
grant execute on procedure users.add_user_role to users_web;

comment on procedure users.add_user_role IS $$
Add a role to a user's list of roles. Note, if the user already has the role, nothing happens.

Arguments:
uid:
    ID specifying the user to add a new role
role:
    Name of the role to add to the specified user
$$;
