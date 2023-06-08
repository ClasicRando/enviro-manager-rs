create or replace procedure workflow_run.set_task_progress(
    workflow_run_id bigint,
    task_order integer,
    progress smallint
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set progress = $3
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::workflow_run.task_status;
$$;

grant execute on procedure workflow_run.set_task_progress to we_web;

comment on procedure workflow_run.set_task_progress IS $$
Set the progress property of a task_queue record.

Arguments:
workflow_run_id:
    ID of the workflow to schedule for running
task_order:
    Task order within the workflow run to be updated
progress:
    Progress percentage of the workflow_run. Must be between 0 and 100
$$;
