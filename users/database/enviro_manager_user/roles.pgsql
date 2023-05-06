create table if not exists enviro_manager_user.roles (
    name text primary key check (data_check.check_not_blank_or_empty(name)),
    description text not null check (data_check.check_not_blank_or_empty(description))
);

call audit.audit_table('enviro_manager_user.roles');

revoke all on enviro_manager_user.roles from public;

comment on table enviro_manager_user.roles is
'All users that will access EnviroManager resources';
comment on column enviro_manager_user.roles.name is
'Unique name for each user role';
comment on column enviro_manager_user.roles.description is
'Long description of what actions the role allows a user to perform';
