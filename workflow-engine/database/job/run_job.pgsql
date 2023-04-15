create or replace procedure job.run_job(job_id bigint)
language plpgsql
as $$
declare
    v_workflow_id bigint;
    v_is_paused boolean;
    v_workflow_run_id bigint;
begin
    begin
        select j.workflow_id, j.is_paused
        into v_workflow_id, v_is_paused
        from job.jobs j
        where j.job_id = $1
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
        select workflow.initialize_workflow_run(v_workflow_id)
        into v_workflow_run_id;

        call workflow.schedule_workflow_run(v_workflow_run_id);

        update job.jobs j
        set
            current_workflow_run_id = v_workflow_run_id,
            next_run = case
                when j.job_type = 'Interval'::job.job_type
                    then j.next_run + j.job_interval
                else job.next_run_job_schedule(j.job_schedule)
            end
        where  j.job_id = $1;
        commit;
    exception
        when others then
            rollback;
            raise;
    end;
end;
$$;

comment on procedure job.run_job IS $$
Attempts to run the job specified. Will fail when:
    - the job_id does not match a record
    - the job is paused
    - error raised when initializing/scheduling/updating the job

!NOTE! This function has transactional controls. If successful, the transaction is committed,
otherwise the transaction will be rolled back before raising/re-raising an exception

Arguments:
job_id:
    ID of the job to run
$$;
