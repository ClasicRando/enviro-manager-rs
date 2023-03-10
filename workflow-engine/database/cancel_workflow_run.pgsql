create or replace procedure workflow_engine.cancel_workflow_run(
    workflow_run_id bigint
)
language sql
as $$
update workflow_engine.workflow_runs wr
set
    status = 'Canceled'::workflow_engine.workflow_run_status,
    executor_id = null
where wr.workflow_run_id = $1;

update task.task_queue tq
set
    status = 'Canceled'::task.task_status,
    task_end = now() at time zone 'UTC',
    output = 'Workflow run canceled'
where
    tq.workflow_run_id = $1
    and tq.status = 'Running'::task.task_status;
$$;

comment on procedure workflow_engine.cancel_workflow_run IS $$
Cancel workflow run by setting workflow run status and updating any running tasks to the 'Canceled'
status with an appropriate output message.

Arguments:
workflow_run_id:
    ID of the workflow to cancel
$$;
