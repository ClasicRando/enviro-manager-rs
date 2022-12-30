create procedure workflow_engine.complete_workflow_run_move(
    workflow_run_id bigint
)
language sql
as $$
update workflow_engine.task_queue
set    status = 'Waiting'::workflow_engine.task_status
where  workflow_run_id = $1
and    task_order = (
    select task_order
    from   workflow_engine.task_queue
    where  workflow_run_id = $1
    and    status = 'Paused'::workflow_engine.task_status
    order by task_order
    limit 1
    for update skip locked
);

update workflow_engine.workflow_runs
set    status = 'Scheduled'::workflow_engine.workflow_run_status
where  workflow_run_id = $1;
$$;

comment on procedure workflow_engine.complete_workflow_run_move IS $$
Finish the transition of a workflow run to another executor. Sets the first 'Paused' task to
'Waiting', then set the workflow run to 'Scheduled' so the next available executor can pick it up.

Arguments:
workflow_run_id:    ID of the workflow run
$$;
