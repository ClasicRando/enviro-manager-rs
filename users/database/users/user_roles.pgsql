create table if not exists users.user_roles (
    em_uid bigint not null references users.users(em_uid)
        on update cascade
        on delete cascade,
    role text not null references users.roles(name)
        on update cascade
        on delete cascade,
    constraint user_roles_pk primary key (em_uid, role)
);

call audit.audit_table('users.user_roles');

revoke all on users.user_roles from public;

comment on table users.user_roles is
'Show the roles a user has within the EnviroManager application suite';
comment on column users.user_roles.em_uid is
'Link to the users table';
comment on column users.user_roles.role is
'Role applied to a user. Links to the roles table';
