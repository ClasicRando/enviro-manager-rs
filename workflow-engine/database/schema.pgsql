create schema if not exists workflow_engine;
revoke all on schema workflow_engine from public;

comment on schema workflow_engine is 'Main area for workflow engine related objects';
