create or replace view users.v_users as
    with user_roles as (
        select ur.uid, array_agg(ur.role)::text[] roles
        from users.user_roles ur
        group by ur.uid
    )
    select
        u.uid, trim(u.first_name) || ' ' || trim(u.last_name) full_name,
        coalesce(ur.roles,'{}'::text[]) roles
    from users.users u
    left join user_roles ur
    on u.uid = ur.uid;

revoke all on users.v_users from public;
grant select on users.v_users to users_web;
