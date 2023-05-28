create or replace procedure task.retry_task(
    workflow_run_id bigint,
    task_order integer
)
security definer
language sql
as $$
update task.task_queue tq
set status = 'Waiting'::task.task_status
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
$$;

grant execute on procedure task.retry_task to we_web;

comment on procedure task.retry_task IS $$
Retry a given task by setting the record to the 'Waiting' status

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to retry
task_order:
    Task order within the workflow run to retry
$$;
