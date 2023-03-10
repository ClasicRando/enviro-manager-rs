create or replace procedure workflow_engine.start_task_run(
    workflow_run_id bigint,
    task_order int
)
language sql
as $$
update task.task_queue tq
set
    status = 'Running'::task.task_status,
    task_start = now() at time zone 'UTC'
where
    tq.workflow_run_id = $1
    and tq.task_order = $2;
$$;

comment on procedure workflow_engine.start_task_run IS $$
Get the next available workflow run for the given executor. Returns at most 1 row of a
workflow_run_id and a flag to indicate if the workflow run is valid or not. Invalid runs are reset
by the executor.

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task_order
task_order:
    Task order within the workflow run to be run
$$;
