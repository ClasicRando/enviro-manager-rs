create or replace procedure workflow_engine.start_workflow_run_move(
    workflow_run_id bigint
)
language sql
as $$
update task.task_queue tq
set status = 'Paused'::task.task_status
where
    tq.workflow_run_id = $1
    and tq.task_order = (
        select tq2.task_order
        from task.task_queue tq2
        where
            tq2.workflow_run_id = $1
            and tq2.status = 'Waiting'::task.task_status
        order by tq2.task_order
        limit 1
        for update skip locked
    );
$$;

comment on procedure workflow_engine.start_workflow_run_move IS $$
Starts the process of transitioning the workflow run to a new executor (or to the 'Scheduled' pool
of workflow runs). Sets the first task_queue record that isn't 'Waiting' to 'Paused'.

Arguments:
workflow_run_id:
    ID of the workflow run
$$;
