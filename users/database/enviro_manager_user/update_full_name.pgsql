create or replace procedure enviro_manager_user.update_username(
    username text,
    password text,
    new_first_name text,
    new_last_name text
)
language plpgsql
as $$
declare
    v_em_uid bigint;
begin
    select u.em_uid
    into strict v_em_uid
    from enviro_manager_user.users u
    where
        u.username = $1
        and u.password = crypt($2, u.password);

    update enviro_manager_user.users u
    set
        first_name = $3,
        last_name = $4
    where u.em_uid = v_em_uid;
exception
    when no_data_found then
        raise exception 'Username or password invalid';
end;
$$;

comment on procedure enviro_manager_user.update_username IS $$
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
