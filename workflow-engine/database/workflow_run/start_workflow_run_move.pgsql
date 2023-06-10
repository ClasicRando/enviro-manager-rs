create or replace procedure workflow_run.start_workflow_run_move(
    workflow_run_id bigint
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set status = 'Paused'::workflow_run.task_status
where
    tq.workflow_run_id = $1
    and tq.task_order = (
        select tq2.task_order
        from workflow_run.task_queue tq2
        where
            tq2.workflow_run_id = $1
            and tq2.status = 'Waiting'::workflow_run.task_status
        order by tq2.task_order
        limit 1
        for update skip locked
    );
$$;

grant execute on procedure workflow_run.start_workflow_run_move to we_web;

comment on procedure workflow_run.start_workflow_run_move IS $$
Starts the process of transitioning the workflow run to a new executor (or to the 'Scheduled' pool
of workflow runs). Sets the first task_queue record that isn't 'Waiting' to 'Paused'.

Arguments:
workflow_run_id:
    ID of the workflow run
$$;
