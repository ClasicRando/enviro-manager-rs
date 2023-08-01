create or replace function users.validate_user(
    username text,
    password text
)
returns table
(
    uid uuid,
    username text,
    full_name text,
    roles text[]
)
immutable
security definer
language sql
as $$
select u.uid, u.username, u.full_name, u.roles
from users.v_users u
where
    u.uid in (
        select u2.uid
        from users.users u2
        where
            u2.username = $1
            and u2.password = crypt($2, u2.password)
    )
$$;

revoke all on function users.validate_user from public;
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
