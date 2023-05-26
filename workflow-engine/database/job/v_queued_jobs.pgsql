create or replace view job.v_queued_jobs as
select j.job_id, j.next_run
from job.jobs j
where 
    not j.is_paused
    and not exists(
        select 1
        from workflow.workflow_runs wr
        where j.current_workflow_run_id = wr.workflow_run_id
        and wr.status != 'Complete'
    )
order by j.next_run;

grant select on view job.v_queued_jobs to we_web;

comment on view job.v_queued_jobs IS $$
Get all jobs that are not paused and is not currently running
$$;
