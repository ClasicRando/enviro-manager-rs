create or replace procedure workflow_run.schedule_workflow_run(
    workflow_run_id bigint,
    executor_id bigint default null
)
language sql
as $$
update workflow_run.workflow_runs wr
set
    status = 'Scheduled'::workflow_run.workflow_run_status,
    executor_id = $2
where
    wr.workflow_run_id = $1
    and wr.status = 'Waiting'::workflow_run.workflow_run_status;
$$;

comment on procedure workflow_run.schedule_workflow_run IS $$
Set workflow run to scheduled with an optional executor_id specified as the intended runner. If not
executor_id specified, the system will choose the executor with minimal load.

Arguments:
workflow_run_id:
    ID of the workflow to schedule for running
executor_id:
    ID of the executor to be manually assigned, default is null (i.e. system decides based upon
    executor distribution)
$$;
