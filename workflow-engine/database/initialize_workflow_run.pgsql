create or replace function workflow_engine.initialize_workflow_run(
    workflow_id bigint
) returns bigint
language plpgsql
as $$
declare
    v_workflow_run_id bigint;
    v_workflow workflow_engine.workflows;
    v_state text;
    v_msg text;
    v_detail text;
    v_hint text;
    v_context text;
begin
    start transaction;
    begin
        select w.workflow_id, w.tasks
        into v_workflow
        from workflow_engine.workflows w
        where w.workflow_id = $1;
    exception
        when no_data_found then
            commit;
            raise exception 'Cannot find a workflow for %l', $1;
    end;

    if v_workflow.deprecated_date is not null then
        commit;
        raise exception 'Cannot initialize a workflow_run with a deprecated workflow. Consider using workflow_id = %l', v_workflow.new_workflow_id;
    end if;

    begin
        insert into workflow_engine.workflow_runs(workflow_id)
        values($1)
        returning workflow_run_id into v_workflow_run_id;

        execute format(
            'create table "workflow_engine".%I partition of workflow_engine.task_queue for values in (%L)',
            'task_queue_'||v_workflow_run_id,
            v_workflow_run_id
        );

        insert into workflow_engine.task_queue(workflow_run_id,task_order,task_id,parameters)
        select v_workflow_run_id, wt.task_order, wt.task_id, wt.parameters
        from workflow_engine.workflow_tasks wt
        join workflow_engine.tasks t on wt.task_id = t.task_id
        where wt.workflow_id = $1;
        commit;
    exception
        when others then
            rollback;
            get stacked diagnostics
                v_state   = returned_sqlstate,
                v_msg     = message_text,
                v_detail  = pg_exception_detail,
                v_hint    = pg_exception_hint,
                v_context = pg_exception_context;

            raise exception E'
                state  : %
                message: %
                detail : %
                hint   : %
                context: %', v_state, v_msg, v_detail, v_hint, v_context;
    end;

    return v_workflow_run_id;
end;
$$;

comment on function workflow_engine.initialize_workflow_run IS $$
Create a new workflow run entry and child tasks in task_queue using the workflow_id provided as a
template.

!NOTE! This function contains transactional logic so when the insert and create partition
statements are completed successfully, everything is committed. If something goes wrong, the
existing transaction if let unaltered.

Arguments:
workflow_id:
    ID of the workflow that is used as a template to build the workflow run
$$;
