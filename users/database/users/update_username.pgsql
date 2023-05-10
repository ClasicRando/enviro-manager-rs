create or replace procedure users.update_username(
    username text,
    password text,
    new_username text
)
language plpgsql
as $$
declare
    v_em_uid bigint;
begin
    select u.em_uid
    into strict v_em_uid
    from users.users u
    where
        u.username = $1
        and u.password = crypt($2, u.password);

    update users.users u
    set username = $3
    where u.em_uid = v_em_uid;
exception
    when no_data_found then
        raise exception 'Username or password invalid';
end;
$$;

comment on procedure users.update_username IS $$
Update an existing user with new username provided. Will raise exception if the username already
exists.

Arguments:
username:
    Unique name of the user to update
password:
    Current password of the user to verify that the update to username is okay
new_username:
    New username to set for the specified user
$$;
