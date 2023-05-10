create or replace procedure users.add_user_role(
    action_em_uid bigint,
    em_uid bigint,
    role text
)
language plpgsql
as $$
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role($1, 'add role');
    insert into users.user_roles(em_uid,role)
    values($2,$3)
    on conflict (name,description) do nothing;
end;
$$;

comment on procedure users.add_user_role IS $$
Add a role to a user's list of roles. Note, if the user already has the role, nothing happens.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
em_uid:
    ID specifying the user to add a new role
role:
    Name of the role to add to the specified user
$$;
