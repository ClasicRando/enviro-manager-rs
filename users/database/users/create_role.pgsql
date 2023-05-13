create or replace procedure users.create_role(
    action_em_uid bigint,
    name text,
    description text
)
language plpgsql
as $$
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role($1, 'create role');
    insert into users.roles(name,description)
    values($2,$3);
end;
$$;

grant execute on procedure users.create_role to users_web;

comment on procedure users.create_role IS $$
Create a new role. Will raise exceptions if the name or description are empty or null.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
name:
    Name of the new role, must be unique within the roles table
description:
    Long description of what actions a role allows a user to perform
$$;