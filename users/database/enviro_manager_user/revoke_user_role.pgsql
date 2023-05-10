create or replace procedure enviro_manager_user.add_user_role(
    action_em_uid bigint,
    em_uid bigint,
    role text
)
language plpgsql
as $$
declare
    v_uid bigint := enviro_manager_user.get_current_em_uid();
begin
    call enviro_manager_user.check_user_role(v_uid, 'add role');
    delete from enviro_manager_user.user_roles ur
    where
        ur.em_uid = $1
        and ur.role = $2;
end;
$$;

comment on procedure enviro_manager_user.add_user_role IS $$
Revoke a role for a specified user.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
em_uid:
    ID specifying the user to revoke a role
role:
    Name of the role to revoke from the specified user
$$;
