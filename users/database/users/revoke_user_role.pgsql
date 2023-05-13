create or replace procedure users.add_user_role(
    action_uid uuid,
    uid uuid,
    role text,
    out uid uuid,
    out full_name text,
    out roles users.roles[]
)
language plpgsql
as $$
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role(v_uid, 'add role');
    delete from users.user_roles ur
    where
        ur.em_uid = $1
        and ur.role = $2;

    select u.uid, u.full_name, u.roles
    into $4, $5, $6
    from users.v_users u
    where u.em_uid = v_em_uid;
end;
$$;

grant execute on procedure users.add_user_role to users_web;

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
