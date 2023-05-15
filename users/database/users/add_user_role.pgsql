create or replace procedure users.add_user_role(
    in_action_uid uuid,
    in_uid uuid,
    in_role text,
    out uid uuid,
    out full_name text,
    out roles users.roles[]
)
language plpgsql
as $$
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role($1, 'add-role');
    insert into users.user_roles(em_uid,role)
    values($2,$3)
    on conflict (name,description) do nothing;
    
    select u.uid, u.full_name, u.roles
    into $4, $5, $6
    from users.v_users u
    where u.em_uid = v_em_uid;
end;
$$;

grant execute on procedure users.add_user_role to users_web;

comment on procedure users.add_user_role IS $$
Add a role to a user's list of roles. Note, if the user already has the role, nothing happens.

Arguments:
action_uid:
    User ID that is attempting to perform the action
uid:
    ID specifying the user to add a new role
role:
    Name of the role to add to the specified user
$$;
