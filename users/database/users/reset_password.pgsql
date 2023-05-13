create or replace procedure users.reset_password(
    username text,
    password text,
    new_password text,
    out uid uuid,
    out full_name text,
    out roles users.roles[]
)
language plpgsql
as $$
declare
    v_em_uid bigint;
begin
    select u.em_uid
    into strict v_em_uid
    from users.validate_user($1, $2);

    update users.users u
    set password = crypt($3, gen_salt('bf'))
    where u.em_uid = v_em_uid;
    
    select u.uid, u.full_name, u.roles
    into $4, $5, $6
    from users.v_users u
    where u.em_uid = v_em_uid;
exception
    when no_data_found then
        raise exception 'Username or password invalid';
end;
$$;

comment on procedure users.reset_password IS $$
Update an existing user with new username provided. Will raise exception if the username already
exists.

Arguments:
username:
    Unique name of the user to update
password:
    Current password of the user to verify that the update to username is okay
new_password:
    New password to set for the specified user
$$;
