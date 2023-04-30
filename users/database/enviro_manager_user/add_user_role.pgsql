create or replace procedure enviro_manager_user.add_user_role(
    em_uid bigint,
    role text
)
language plpgsql
as $$
declare
    v_uid bigint := enviro_manager_user.get_current_em_uid();
begin
    call enviro_manager_user.check_user_role(v_uid, 'add role');
    insert into enviro_manager_user.user_roles(em_uid,role)
    values($1,$2)
    on conflict (name,description) do nothing;
end;
$$;

comment on procedure enviro_manager_user.add_user_role IS $$
Add a role to a user's list of roles. Note, if the user already has the role, nothing happens.

Arguments:
em_uid:
    ID specifying the user to add a new role
role:
    Name of the role to add to the specified user
$$;
