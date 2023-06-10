create or replace procedure workflow_run.cancel_workflow_run(
    workflow_run_id bigint
)
security definer
language sql
as $$
update workflow_run.workflow_runs wr
set
    status = 'Canceled'::workflow_run.workflow_run_status,
    executor_id = null
where wr.workflow_run_id = $1;

update workflow_run.task_queue tq
set
    status = 'Canceled'::workflow_run.task_status,
    task_end = now() at time zone 'UTC',
    output = 'Workflow run canceled'
where
    tq.workflow_run_id = $1
    and tq.status = 'Running'::workflow_run.task_status;
$$;

grant execute on procedure workflow_run.cancel_workflow_run to we_web;

comment on procedure workflow_run.cancel_workflow_run IS $$
Cancel workflow run by setting workflow run status and updating any running tasks to the 'Canceled'
status with an appropriate output message.

Arguments:
workflow_run_id:
    ID of the workflow to cancel
$$;
