create or replace procedure workflow_engine.complete_workflow_run_move(
    workflow_run_id bigint
)
language sql
as $$
begin;
update workflow_engine.task_queue tq
set status = 'Waiting'::workflow_engine.task_status
where 
    tq.workflow_run_id = $1
    and tq.task_order = (
        select tq2.task_order
        from workflow_engine.task_queue tq2
        where
            tq2.workflow_run_id = $1
            and tq2.status = 'Paused'::workflow_engine.task_status
        order by tq2.task_order
        limit 1
        for update skip locked
    );

update workflow_engine.workflow_runs wr
set status = 'Scheduled'::workflow_engine.workflow_run_status
where wr.workflow_run_id = $1;
commit;
$$;

comment on procedure workflow_engine.complete_workflow_run_move IS $$
Finish the transition of a workflow run to another executor. Sets the first 'Paused' task to
'Waiting', then set the workflow run to 'Scheduled' so the next available executor can pick it up.

Arguments:
workflow_run_id:
    ID of the workflow run
$$;
