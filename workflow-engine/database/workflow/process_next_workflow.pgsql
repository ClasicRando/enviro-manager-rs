create or replace procedure workflow.process_next_workflow(
    executor_id bigint,
    out workflow_run_id bigint
)
language plpgsql
as $$
declare
    v_is_valid boolean;
begin
    start transaction;
    begin
        select w.workflow_run_id, w.is_valid
        into strict $2, v_is_valid
        from workflow.next_workflow($1) w
        limit 1;

        if v_is_valid then
            call workflow.start_workflow_run($2, $1);
        else
            call workflow.complete_workflow_run($2);
        end if;
        commit;
    exception
        when no_data_found then
            commit;
            return;
        when others then
            rollback;
            raise;
    end;
end;
$$;

comment on procedure workflow.process_next_workflow IS $$
Process the next available workflow run for the given executor. Checks the next available workflow,
starting or completing the workflow depending on the is_valid flag collected. Returns the
workflow_run_id of the next workflow if it was started successfully. Otherwise returns null.

Arguments:
executor_id:
    ID of the executor to filter workflow runs (i.e. do not pick up workflow runs marked for
    another executor)
$$;
