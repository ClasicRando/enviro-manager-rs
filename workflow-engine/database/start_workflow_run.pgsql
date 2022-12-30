create or replace procedure workflow_engine.start_workflow_run(
    workflow_run_id bigint,
    executor_id bigint
)
language sql
as $$
update workflow_engine.workflow_runs
set    status = 'Running'::workflow_engine.workflow_run_status,
       executor_id = $2,
       progress = 0
where  workflow_run_id = $1;
$$;

comment on procedure workflow_engine.start_workflow_run IS $$
Start the workflow run by setting the status and owner executor

Arguments:
workflow_run_id:    ID of workflow run that is starting
executor_id:        ID of executor that is running this workflow run
$$;
