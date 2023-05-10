create or replace procedure users.add_user_role(
    action_em_uid bigint,
    em_uid bigint,
    role text
)
language plpgsql
as $$
declare
    v_uid bigint := users.get_current_em_uid();
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role(v_uid, 'add role');
    delete from users.user_roles ur
    where
        ur.em_uid = $1
        and ur.role = $2;
end;
$$;

comment on procedure users.add_user_role IS $$
Revoke a role for a specified user.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
em_uid:
    ID specifying the user to revoke a role
role:
    Name of the role to revoke from the specified user
$$;
