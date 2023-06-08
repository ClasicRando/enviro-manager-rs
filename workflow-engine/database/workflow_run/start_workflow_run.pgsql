create or replace procedure workflow_run.start_workflow_run(
    workflow_run_id bigint,
    executor_id bigint
)
security definer
language sql
as $$
update workflow_run.workflow_runs wr
set
    status = 'Running'::workflow_run.workflow_run_status,
    executor_id = $2,
    progress = 0
where wr.workflow_run_id = $1;
$$;

grant execute on procedure workflow_run.start_workflow_run to we_web;

comment on procedure workflow_run.start_workflow_run IS $$
Start the workflow run by setting the status and owner executor

Arguments:
workflow_run_id:
    ID of workflow run that is starting
executor_id:
    ID of executor that is running this workflow run
$$;
