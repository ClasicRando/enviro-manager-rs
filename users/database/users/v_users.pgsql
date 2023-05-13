create or replace view users.v_users as
    with user_roles as (
        select ur.em_uid, array_agg(r.*)::users.roles[] roles
        from users.user_roles ur
        join users.roles r
        on ur.role = r.name
        group by ur.em_uid
    )
    select ur.em_uid, u.uid, trim(u.first_name) || ' ' || trim(u.last_name) full_name, ur.roles
    from users.users u
    join user_roles ur
    on u.em_uid = ur.em_uid;