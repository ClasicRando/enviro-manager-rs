create table workflow_engine.workflows (
    workflow_id bigint primary key generated always as identity,
    name text not null check(data_check.check_not_blank_or_empty(name)) unique,
    is_deprecated boolean not null default false,
    new_workflow bigint references workflow_engine.workflows match simple
        on delete null
        on update cascade,
    constraint deprecation_check check (
        case when new_workflow is not null then is_deprecated else true end
    )
);

call audit.audit_table('workflow_engine.workflows');

revoke all on workflow_engine.workflows from public;

comment on table workflow_engine.workflows is 'All workflows able to be executed by the workflow engine. Tasks are identified in a child table';
comment on column workflow_engine.workflows.workflow_id is 'Unique identifier for each workflow';
comment on column workflow_engine.workflows.name is 'Alias given to the workflow. Usually describes the process executed. Must be unique';
comment on column workflow_engine.workflows.is_deprecated is 'Flag indicating that the workflow should no longer be used. Check audit table for date of deprecation';
comment on column workflow_engine.workflows.new_workflow is 'Workflow_id of the workflow that replaced this workflow';
comment on constraint workflow_engine.workflows.deprecation_check is 'Check to ensure that a new workflow id is provided only when the is_deprecated flag is true';
