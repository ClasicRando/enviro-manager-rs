create or replace view users.v_roles as
    select r.name, r.description
    from users.roles r;

revoke all on users.v_roles from public;
grant select on users.v_roles to users_web;
