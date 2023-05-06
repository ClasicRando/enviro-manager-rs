create or replace procedure enviro_manager_user.create_role(
    name text,
    description text
)
language plpgsql
as $$
declare
    v_uid bigint := enviro_manager_user.get_current_em_uid();
begin
    call enviro_manager_user.check_user_role(v_uid, 'create role');
    insert into enviro_manager_user.roles(name,description)
    values($1,$2);
end;
$$;

comment on procedure enviro_manager_user.create_role IS $$
Create a new role. Will raise exceptions if the name or description are empty or null.

Arguments:
name:
    Name of the new role, must be unique within the roles table
description:
    Long description of what actions a role allows a user to perform
$$;
