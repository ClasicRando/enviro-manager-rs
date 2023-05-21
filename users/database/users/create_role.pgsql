create or replace function users.create_role(
    in_name text,
    in_description text,
    out name text,
    out description text
)
returns record
returns null on null input
volatile
language sql
as $$
insert into users.roles as r (name,description)
values($1,$2)
returning r.name, r.description;
$$;

revoke all on function users.create_role from public;
grant execute on function users.create_role to users_web;

comment on function users.create_role IS $$
Create a new role

Arguments:
action_uid:
    User ID that is attempting to perform the action
name:
    Name of the new role, must be unique within the roles table
description:
    Long description of what actions a role allows a user to perform
$$;
