if current_database() like '%test%' then
    create schema if not exists job authorization we_test;
else
    create schema if not exists job authorization we_admin;
end if;
revoke all on schema job from public;
grant usage on schema job to we_web;
comment on schema job is 'Job related objects for the workflow engine';

if current_database() like '%test%' then
    create schema if not exists task authorization we_test;
else
    create schema if not exists task authorization we_admin;
end if;
revoke all on schema task from public;
grant usage on schema task to we_web;
comment on schema task is 'Task related objects for the workflow engine';

if current_database() like '%test%' then
    create schema if not exists executor authorization we_test;
else
    create schema if not exists executor authorization we_admin;
end if;
revoke all on schema executor from public;
grant usage on schema executor to we_web;
comment on schema executor is 'Executor related objects for the workflow engine';

if current_database() like '%test%' then
    create schema if not exists workflow authorization we_test;
else
    create schema if not exists workflow authorization we_admin;
end if;
revoke all on schema workflow from public;
grant usage on schema workflow to we_web;
comment on schema workflow is 'Workflow related objects for the workflow engine';

grant usage on schema data_check to we_web;
grant usage on schema audit to we_web;

