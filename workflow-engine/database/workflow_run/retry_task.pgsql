create or replace procedure workflow_run.retry_task(
    workflow_run_id bigint,
    task_order integer
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set status = 'Waiting'::workflow_run.task_status
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
$$;

grant execute on procedure workflow_run.retry_task to we_web;

comment on procedure workflow_run.retry_task IS $$
Retry a given task by setting the record to the 'Waiting' status

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to retry
task_order:
    Task order within the workflow run to retry
$$;
