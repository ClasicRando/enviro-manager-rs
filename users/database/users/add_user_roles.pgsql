create or replace procedure users.add_user_roles(
    uid uuid,
    roles text[]
)
language sql
as $$
insert into users.user_roles(uid, role)
select u.uid, r.description
from users.users u
cross join unnest($2) r(description)
where u.uid = $1
on conflict (uid, role) do nothing;
$$;

revoke all on procedure users.add_user_roles from public;
grant execute on procedure users.add_user_roles to users_web;

comment on procedure users.add_user_roles IS $$
Add a list of roles to the user. Note, if a role already exists then nothing will happen

Arguments:
uid:
    UUID specifying the user to add a new role
roles:
    Roles to be added to the user as an array of role names
$$;
