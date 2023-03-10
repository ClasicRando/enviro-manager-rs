create or replace procedure workflow_engine.complete_task(
    workflow_run_id bigint,
    task_order integer
)
language plpgsql
as $$
begin
    if exists(
        select 1
        from task.task_queue tq
        where
            tq.workflow_run_id = $1
            and tq.task_order = $2
            and tq.status != 'Paused'::task.task_status
    ) then
        raise 'Cannot complete task. Status must be "Paused"';
    end if;

    update task.task_queue tq
    set
        status = 'Complete'::task.task_status,
        progress = 100
    where
        tq.workflow_run_id = $1
        and tq.task_order = $2;

    with tasks as (
        select
            tq.workflow_run_id,
            count(0) filter (where tq.status = 'Complete'::task.task_status) complete_count,
            count(0) total_tasks
        from task.task_queue tq
        group by tq.workflow_run_id
    )
    update workflow_engine.workflow_runs wr
    set
        status = 'Scheduled'::workflow_engine.workflow_run_status,
        progress = round((t.complete_count / cast(t.total_tasks as real)) * 100)::smallint
    from tasks t
    where
        wr.workflow_run_id = t.workflow_run_id
        and wr.workflow_run_id = $1;
end;
$$;

comment on procedure workflow_engine.complete_task IS $$
Manually complete a task that is paused to continue with workflow run.

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to complete
task_order:
    Task order within the workflow run to complete
$$;
