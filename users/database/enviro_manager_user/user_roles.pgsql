create table if not exists enviro_manager_user.user_roles (
    em_uid bigint not null references enviro_manager_user.users(em_uid)
        on update cascade
        on delete cascade,
    role text not null references enviro_manager_user.roles(name)
        on update cascade
        on delete cascade,
    constraint user_roles_pk primary key (em_uid, role)
);

call audit.audit_table('enviro_manager_user.user_roles');

revoke all on enviro_manager_user.user_roles from public;

comment on table enviro_manager_user.user_roles is
'Show the roles a user has within the EnviroManager application suite';
comment on column enviro_manager_user.user_roles.em_uid is
'Link to the users table';
comment on column enviro_manager_user.user_roles.role is
'Role applied to a user. Links to the roles table';
