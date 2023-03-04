create or replace procedure workflow_engine.restart_workflow_run(
    workflow_run_id bigint
)
language plpgsql
as $$
begin
    start transaction;
    if exists(
        select 1
        from workflow_engine.workflow_runs t1
        where
            t1.workflow_run_id = $1
            and t1.status = 'Running'::workflow_engine.workflow_run_status
    ) then
        rollback;
        raise exception using message = 'Cannot restart a workflow run that is in progress. ' ||
            'Please cancel the workflow run before restarting';
    end if;

    begin
        insert into workflow_engine.task_queue_archive(
            workflow_run_id,task_order,task_id,status,parameters,output,rules,task_start,task_end
        )
        select
            tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, tq.output,
            tq.rules, tq.task_start, tq.task_end
        from workflow_engine.task_queue tq
        where tq.workflow_run_id = $1
        for update;

        update workflow_engine.task_queue tq
        set
            status = 'Waiting'::workflow_engine.task_status,
            output = null,
            task_start = null,
            task_end = null
        where tq.workflow_run_id = $1;

        update workflow_engine.workflow_runs wr
        set
            status = 'Waiting'::workflow_engine.workflow_run_status,
            executor_id = null
        where wr.workflow_run_id = $1;
        commit;
    exception
        when others then
            rollback;
            raise;
    end;
end;
$$;

comment on procedure workflow_engine.restart_workflow_run IS $$
Restart a given workflow run if possible. Archives the current state of the tasks and then updates
all the tasks and workflow_run record.

!NOTE! This procedure auto commits if successful or performs a rollback if the archive/update
commands are not successful.  

Arguments:
workflow_run_id:
    ID of the workflow run to be restarted
$$;
