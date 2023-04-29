create or replace view enviro_manager_user.v_users as
    with user_roles as (
        select ur.em_uid, array_agg(r.*)::enviro_manager_user.roles[] roles
        from enviro_manager_user.user_roles ur
        join enviro_manager_user.roles r
        on ur.role = r.name
        group by ur.em_uid
    )
    select u.em_uid, trim(u.first_name) || ' ' || trim(u.last_name) full_name, ur.roles
    from enviro_manager_user.users u
    join user_roles ur
    on u.em_uid = ur.em_uid;