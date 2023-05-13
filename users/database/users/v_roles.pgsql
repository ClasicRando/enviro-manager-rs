create or replace view users.v_roles as
    select r.name, r.description
    from users.roles r;

grant select on users.v_roles to users_web;
