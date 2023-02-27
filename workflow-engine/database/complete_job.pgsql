create or replace function workflow_engine.complete_job(
    job_id bigint
) returns text
language plpgsql
as $$
declare
    v_workflow_run_id bigint;
    v_status workflow_engine.workflow_run_status;
    v_is_paused boolean;
    v_state text;
    v_msg text;
    v_detail text;
    v_hint text;
    v_context text;
begin
    begin
        select j.current_workflow_run_id
        into v_workflow_run_id
        from workflow_engine.jobs j
        where j.job_id = $1
        for update;
    exception
        when no_data_found then
            rollback;
            return format('No job for job_id = %.', $1);
    end;

    if v_workflow_run_id is null then
        rollback;
        return 'Job must be active to finish';
    end if;

    select wr.status
    into v_status
    from workflow_engine.workflow_runs wr
    where wr.workflow_run_id = v_workflow_run_id;

    if v_status in (
        'Scheduled'::workflow_engine.workflow_run_status,
        'Running'::workflow_engine.workflow_run_status
    ) then
        rollback;
        return 'Workflow must be done to complete job';
    end if;

    begin
        update workflow_engine.jobs j
        set
            current_workflow_run_id = case
                when v_status = 'Complete'::workflow_engine.workflow_run_status then null
                else current_workflow_run_id
            end,
            is_paused = case
                when v_status = 'Complete'::workflow_engine.workflow_run_status then false
                else true
            end
        where j.job_id = $1
        returning is_paused into v_is_paused;
    exception
        when others then
            get stacked diagnostics
                v_state   = returned_sqlstate,
                v_msg     = message_text,
                v_detail  = pg_exception_detail,
                v_hint    = pg_exception_hint,
                v_context = pg_exception_context;

            return format(E'Exception raised during job update
                state  : %
                message: %
                detail : %
                hint   : %
                context: %', v_state, v_msg, v_detail, v_hint, v_context);
    end;

    commit;

    return case
        when v_is_paused then format('Paused job due to issue with workflow run = %', v_workflow_run_id)
        else ''
    end;
end;
$$;

comment on function workflow_engine.complete_job IS $$
Attempts to complete the job specified. Will return an error message when:
    - the job_id does not match a record
    - the job is not active
    - the current workflow run was not successful
    - error raised when updating the job
    - the job is paused due to a workflow run issue

!NOTE! This function has transactional controls. If successful, the transaction is committed,
otherwise the transaction will be rolled back before returning a message.

Arguments:
job_id:
    ID of the job to run
$$;
