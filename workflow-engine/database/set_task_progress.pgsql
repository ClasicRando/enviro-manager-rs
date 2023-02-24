create or replace procedure workflow_engine.set_task_progress(
    workflow_run_id bigint,
    task_order integer,
    progress smallint
)
language sql
as $$
update workflow_engine.task_queue tq
set progress = $3
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::workflow_engine.task_status;
$$;

comment on procedure workflow_engine.set_task_progress IS $$
Set the progress property of a task_queue record.

Arguments:
workflow_run_id:
    ID of the workflow to schedule for running
task_order:
    Task order within the workflow run to be updated
progress:
    Progress percentage of the task. Must be between 0 and 100
$$;
