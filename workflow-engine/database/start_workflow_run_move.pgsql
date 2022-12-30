create procedure workflow_engine.start_workflow_run_move(
    workflow_run_id bigint
)
language sql
as $$
update workflow_engine.task_queue
set    status = 'Paused'::workflow_engine.task_status
where  workflow_run_id = $1
and    task_order = (
    select task_order
    from   workflow_engine.task_queue
    where  workflow_run_id = $1
    and    status = 'Waiting'::workflow_engine.task_status
    order by task_order
    limit 1
    for update skip locked
);
$$;

comment on procedure workflow_engine.start_workflow_run_move IS $$
Starts the process of transitioning the workflow run to a new executor (or to the 'Scheduled' pool
of workflow runs). Sets the first task_queue record that isn't 'Waiting' to 'Paused'.

Arguments:
workflow_run_id:    ID of the workflow run
$$;
