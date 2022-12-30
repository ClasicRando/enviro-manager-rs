create or replace procedure workflow_engine.complete_task(
    workflow_run_id bigint,
    task_order integer
)
language plpgsql
as $$
begin
    if exists(
        select 1
        from   workflow_engine.task_queue tq
        where  tq.workflow_run_id = $1
        and    tq.task_order = $2
        and    tq.status != 'Paused'::workflow_engine.task_status
    ) then
        raise 'Cannot complete task. Status must be "Paused"';
    end if;

    update workflow_engine.task_queue tq
    set    status = 'Complete'::workflow_engine.task_status
    where  tq.workflow_run_id = $1
    and    tq.task_order = $2;

    update workflow_engine.workflow_runs wr
    set    status = 'Scheduled'::workflow_engine.workflow_run_status
    where  wr.workflow_run_id = $1;
end;
$$;

comment on procedure workflow_engine.complete_task IS $$
Manually complete a task that is paused to continue with workflow run.

Arguments:
workflow_run_id:    ID of the workflow run that owns the task to complete
task_order:         Task order within the workflow run to complete
$$;
