create or replace procedure enviro_manager_user.create_user(
    first_name text,
    last_name text,
    username text,
    password text,
    roles text[],
    out em_uid bigint,
    out full_name text,
    out roles2 enviro_manager_user.roles[]
)
language plpgsql
as $$
declare
    v_uid bigint;
begin
    call enviro_manager_user.validate_password($4);

    insert into enviro_manager_user.users(first_name,last_name,username,password)
    values($1,$2,$3,crypt($4, gen_salt('bf')))
    returning em_uid into v_uid;

    begin
        insert into enviro_manager_user.user_roles(em_uid,role)
        select v_uid, d.name
        from unnest($5) d(name);
    exception
        when others then
            rollback;
            raise;
    end;

    select u.em_uid, u.full_name, u.roles
    into $6, $7, $8
    from enviro_manager_user.v_users u
    where u.em_uid = v_uid;
end;
$$;

comment on procedure enviro_manager_user.create_user IS $$
Create a new user with the provided details, returning the new user data if successful. Will raise
exceptions if the password is invalid or any role entry does not match an existing role type.

Arguments:
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
