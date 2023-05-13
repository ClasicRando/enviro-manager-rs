create or replace procedure users.update_full_name(
    username text,
    password text,
    new_first_name text,
    new_last_name text,
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
    set
        first_name = $3,
        last_name = $4
    where u.em_uid = v_em_uid;

    select u.uid, u.full_name, u.roles
    into $5, $6, $7
    from users.v_users u
    where u.em_uid = v_em_uid;
exception
    when no_data_found then
        raise exception 'Username or password invalid';
end;
$$;

grant execute on procedure users.update_full_name to users_web;

comment on procedure users.update_full_name IS $$
Update an existing user with new username provided. Will raise exception if the username already
exists.

Arguments:
username:
    Unique name of the user to update
password:
    Current password of the user to verify that the update to username is okay
new_first_name:
    New first name to set for the specified user
new_last_name:
    New first name to set for the specified user
$$;
