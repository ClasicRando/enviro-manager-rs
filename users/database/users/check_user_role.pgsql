create or replace procedure users.check_user_role(
    uid uuid,
    role text
)
language plpgsql
as $$
begin
    if current_user != 'users_admin' and not exists(
        select 1
        from users.users u
        join users.user_roles ur
        on u.em_uid = ur.em_uid
        where
            u.uid = $1
            and ur.role = $2
        union all
        select 1
        from users.users u
        join users.user_roles ur
        on u.em_uid = ur.em_uid
        where
            u.uid = $1
            and ur.role = 'admin'
    ) then
        raise exception using message =
            'User ID = ' || $1::text || ' does not have the required role, "' || $2 || '"';
    end if;
end;
$$;

comment on procedure users.check_user_role IS $$
Checks to ensure the specified user has the provided role. If not, an exception is raised. Note, if
the current user is the admin user, then this procedure never fails.

Arguments:
em_uid:
    ID of the user to verify a required role
role:
    Name of the role to verify
$$;
