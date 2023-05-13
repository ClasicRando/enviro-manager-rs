create schema if not exists users authorization users_admin;
revoke all on schema users from public;
comment on schema users is 'Main area for EnviroManager User related objects';
