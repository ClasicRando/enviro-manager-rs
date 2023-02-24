create or replace procedure workflow_engine.retry_task(
    workflow_run_id bigint,
    task_order integer
)
language plpgsql
as $$
begin
    start transaction;
    if exists(
        select 1
        from workflow_engine.task_queue tq
        where
            tq.workflow_run_id = $1
            and tq.task_order = $2
            and tq.status not in (
                'Failed'::workflow_engine.task_status,
                'Rule Broken'::workflow_engine.task_status
            )
    ) then
        rollback;
        raise exception
            'Cannot retry task. Status must be "Failed" or "Rule Broken"';
    end if;

    begin
        insert into workflow_engine.task_queue_archive(
            workflow_run_id,task_order,task_id,status,parameters,output,rules,task_start,task_end
        )
        select
            tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, tq.output,
            tq.rules, tq.task_start, tq.task_end
        from   workflow_engine.task_queue tq
        where 
            tq.workflow_run_id = $1
            and tq.task_order = $2
        for update;

        update workflow_engine.task_queue tq
        set status = 'Waiting'::workflow_engine.task_status
        where
            tq.workflow_run_id = $1
            and tq.task_order = $2;

        update workflow_engine.workflow_runs wr
        set status = 'Scheduled'::workflow_engine.workflow_run_status
        where wr.workflow_run_id = $1;
        commit;
    exception
        when others then
            rollback;
            raise;
    end;
end;
$$;

comment on procedure workflow_engine.retry_task IS $$
Retry a given task if possible. Archives the current state of the task and then updates the task
and the parent workflow_run record.

!NOTE! This procedure auto commits if successfull or performs a rollback if the archive/update
commands are not successfull.  

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to retry
task_order:
    Task order within the workflow run to retry
$$;
