create schema if not exists workflow_engine authorization workflow_engine_admin;
revoke all on schema workflow_engine from public;
grant usage on schema workflow_engine to workflow_engine_user;
comment on schema workflow_engine is 'Main area for workflow engine related objects';

create schema if not exists job authorization workflow_engine_admin;
revoke all on schema job from public;
grant usage on schema job to workflow_engine_user;
comment on schema workflow_engine is 'Job related objects for the workflow engine';

alter schema audit owner to workflow_engine_admin;
revoke all on schema audit from public;
alter schema data_check owner to workflow_engine_admin;
revoke all on schema data_check from public;
grant usage on schema data_check to workflow_engine_user;
grant usage on schema audit to workflow_engine_user;

