create or replace procedure workflow_engine.run_job(job_id bigint)
language plpgsql
as $$
declare
    v_workflow_id bigint;
    v_is_paused boolean;
    v_workflow_run_id bigint;
begin
    begin
        select workflow_id, is_paused
        into   v_workflow_id, v_is_paused
        from   workflow_engine.jobs
        where  job_id = $1
        for update;
    exception
        when no_data_found then
            rollback;
            raise exception 'No job for job_id = %', $1;
    end;

    if v_is_paused then
        rollback;
        raise exception 'Jobs that are currently paused cannot be run';
    end if;

    begin
        select workflow_engine.initialize_workflow_run(v_workflow_id)
        into   v_workflow_run_id;

        call workflow_engine.schedule_workflow_run(v_workflow_run_id);

        update workflow_engine.jobs j
        set    current_workflow_run_id = v_workflow_run_id,
               next_run = case
                            when j.job_type = 'Interval'::workflow_engine.job_type then j.next_run + j.job_interval
                            else workflow_engine.next_run_job_schedule(j.job_schedule)
                          end
        where  j.job_id = $1;
    exception
        when others then
            rollback;
            raise;
    end;

    commit;
end;
$$;

comment on procedure workflow_engine.run_job IS $$
Attempts to run the job specified. Will fail when:
    - the job_id does not match a record
    - the job is paused
    - error raised when initializing/scheduling/updating the job

!NOTE! This function has transactional controls. If successfull, the transaction is committed,
otherwise the transaction will be rolled back before raising/re-raising an exception

Arguments:
job_id:    ID of the job to run
$$;
