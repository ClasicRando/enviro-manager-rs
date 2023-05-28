create or replace procedure task.fail_task_run(
    workflow_run_id bigint,
    task_order integer,
    message text
)
security definer
language sql
as $$
update task.task_queue tq
set
    status = 'Failed'::task.task_status,
    output = $3,
    task_end = now() at time zone 'UTC'
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::task.task_status;
$$;

grant execute on procedure task.fail_task_run to we_web;

comment on procedure task.fail_task_run IS $$
Set the task record as 'Failed' with a required message to explain the failure

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task
task_order:
    Task order within the workflow run to be run
output:
    Message output from the task run, must be non-null and not empty
$$;
