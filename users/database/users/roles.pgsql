create table if not exists users.roles (
    name text primary key check (data_check.check_not_blank_or_empty(name)),
    description text not null check (data_check.check_not_blank_or_empty(description))
);

call audit.audit_table('users.roles');

revoke all on users.roles from public;

comment on table users.roles is
'All users that will access EnviroManager resources';
comment on column users.roles.name is
'Unique name for each user role';
comment on column users.roles.description is
'Long description of what actions the role allows a user to perform';
