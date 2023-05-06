create schema if not exists enviro_manager_user authorization emu_admin;
revoke all on schema enviro_manager_user from public;
grant usage on schema enviro_manager_user to emu_user;
comment on schema enviro_manager_user is 'Main area for EnviroManager User related objects';
