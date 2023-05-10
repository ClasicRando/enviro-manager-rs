create or replace procedure enviro_manager_user.create_role(
    action_em_uid bigint,
    name text,
    description text
)
language plpgsql
as $$
begin
    call enviro_manager_user.check_user_role($1, 'create role');
    insert into enviro_manager_user.roles(name,description)
    values($2,$3);
end;
$$;

comment on procedure enviro_manager_user.create_role IS $$
Create a new role. Will raise exceptions if the name or description are empty or null.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
name:
    Name of the new role, must be unique within the roles table
description:
    Long description of what actions a role allows a user to perform
$$;
