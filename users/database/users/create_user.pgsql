create or replace procedure users.create_user(
    action_em_uid bigint,
    first_name text,
    last_name text,
    username text,
    password text,
    roles text[],
    out em_uid bigint,
    out full_name text,
    out roles2 users.roles[]
)
language plpgsql
as $$
declare
    v_em_uid bigint;
begin
    perform set_config('em.uid',$1::text,false);
    call users.check_user_role($1, 'create user');
    call users.validate_password($5);

    insert into users.users(first_name,last_name,username,password)
    values($2,$3,$4,crypt($5, gen_salt('bf')))
    returning em_uid into v_em_uid;

    begin
        insert into users.user_roles(em_uid,role)
        select v_em_uid, d.name
        from unnest($6) d(name);
    exception
        when others then
            rollback;
            raise;
    end;

    select u.em_uid, u.full_name, u.roles
    into $7, $8, $9
    from users.v_users u
    where u.em_uid = v_em_uid;
end;
$$;

comment on procedure users.create_user IS $$
Create a new user with the provided details, returning the new user data if successful. Will raise
exceptions if the password is invalid or any role entry does not match an existing role type.

Arguments:
action_em_uid:
    User ID that is attempting to perform the action
first_name:
    First name of the new user
last_name:
    Last name of the new user
username:
    Username of the new user
password:
    Password of the new user, validated as the first step
roles:
    Roles of the new user as an array of role names. Validation is performed from the existing
    foreign key when attempting to insert
$$;
