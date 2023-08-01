create or replace view users.v_users as
    with user_roles as (
        select ur.uid, array_agg(ur.role)::text[] roles
        from users.user_roles ur
        group by ur.uid
    )
    select u.uid, u.username, u.full_name, coalesce(ur.roles,'{}'::text[]) roles
    from users.users u
    left join user_roles ur
    on u.uid = ur.uid;

grant select on users.v_users to users_web;
