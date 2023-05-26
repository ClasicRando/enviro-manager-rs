create table if not exists users.user_roles (
    uid uuid not null references users.users(uid)
        on update cascade
        on delete cascade,
    role text not null check (data_check.check_not_blank_or_empty(role)),
    constraint user_roles_pk primary key (uid, role)
);

call audit.audit_table('users.user_roles');

comment on table users.user_roles is
'Show the roles a user has within the EnviroManager application suite';
comment on column users.user_roles.uid is
'Link to the users table';
comment on column users.user_roles.role is
'Role applied to a user. Links to the roles table';
