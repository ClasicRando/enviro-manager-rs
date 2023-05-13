create or replace function users.validate_user(
    username text,
    password text,
    out em_uid bigint,
    out full_name text,
    out roles users.roles[]
)
returns record
returns null on null input
immutable
language sql
as $$
select u.em_uid, u.uid, u.full_name, u.roles
from users.v_users u
where
    u.em_uid in (
        select u2.em_uid
        from users.users u2
        where
            u2.username = $1
            and u2.password = crypt($2, u2.password)
    )
$$;

grant execute on function users.validate_user to users_web;

comment on function users.validate_user IS $$
Validates that the credentials passed in match a user. If the user is found, then it returns the
user ID, name and the roles of the user.

Arguments:
username:
    Username of the user to validate
password:
    Password of the user to validate
$$;
