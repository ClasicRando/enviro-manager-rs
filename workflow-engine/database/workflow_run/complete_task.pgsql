create or replace procedure workflow_run.complete_task(
    workflow_run_id bigint,
    task_order integer
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set
    status = 'Complete'::workflow_run.task_status,
    progress = 100
where
    tq.workflow_run_id = $1
    and tq.task_order = $2;
$$;

grant execute on procedure workflow_run.complete_task to we_web;

comment on procedure workflow_run.complete_task IS $$
Manually complete a task that is paused to continue with workflow run.

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to complete
task_order:
    Task order within the workflow run to complete
$$;
