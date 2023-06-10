create or replace procedure job.set_job_as_running(
    job_id bigint,
    workflow_run_id bigint
)
security definer
language sql
as $$
update job.jobs j
set
    current_workflow_run_id = $2,
    next_run = case
        when j.job_type = 'Interval'::job.job_type
            then j.next_run + j.job_interval
        else job.next_run_job_schedule(j.job_schedule)
    end
where  j.job_id = $1
$$;

grant execute on procedure job.set_job_as_running to we_web;

comment on procedure job.set_job_as_running IS $$


Arguments:
job_id:
    ID of the job to run
$$;
