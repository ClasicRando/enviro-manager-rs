create schema if not exists workflow_engine authorization workflow_engine_admin;
revoke all on schema workflow_engine from public;
grant usage on schema workflow_engine to workflow_engine_user;
comment on schema workflow_engine is 'Main area for workflow engine related objects';

create schema if not exists job authorization workflow_engine_admin;
revoke all on schema job from public;
grant usage on schema job to workflow_engine_user;
comment on schema job is 'Job related objects for the workflow engine';

create schema if not exists task authorization workflow_engine_admin;
revoke all on schema task from public;
grant usage on schema task to workflow_engine_user;
comment on schema task is 'Task related objects for the workflow engine';

create schema if not exists executor authorization workflow_engine_admin;
revoke all on schema executor from public;
grant usage on schema executor to workflow_engine_user;
comment on schema executor is 'Executor related objects for the workflow engine';

create schema if not exists workflow authorization workflow_engine_admin;
revoke all on schema workflow from public;
grant usage on schema workflow to workflow_engine_user;
comment on schema workflow is 'Workflow related objects for the workflow engine';

alter schema audit owner to workflow_engine_admin;
revoke all on schema audit from public;
alter schema data_check owner to workflow_engine_admin;
revoke all on schema data_check from public;
grant usage on schema data_check to workflow_engine_user;
grant usage on schema audit to workflow_engine_user;

