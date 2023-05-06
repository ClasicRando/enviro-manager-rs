create or replace function enviro_manager_user.validate_user(
    username text,
    password text,
    out em_uid bigint,
    out full_name text,
    out roles enviro_manager_user.roles[]
)
returns record
returns null on null input
immutable
language sql
as $$
select u.em_uid, u.full_name, u.roles
from enviro_manager_user.v_users u
where
    u.em_uid in (
        select u2.em_uid
        from enviro_manager_user.users u2
        where
            u2.username = $1
            and u2.password = crypt($2, u2.password)
    )
$$;

comment on function enviro_manager_user.validate_user IS $$
Validates that the credentials passed in match a user. If the user is found, then it returns the
user ID, name and the roles of the user.

Arguments:
username:
    Username of the user to validate
password:
    Password of the user to validate
$$;
