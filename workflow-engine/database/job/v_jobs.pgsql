create or replace view job.v_jobs as
    select
        j.job_id, j.workflow_id, w.name workflow_name, j.job_type,
        j.maintainer, j.job_schedule, j.job_interval, j.is_paused, j.next_run,
        j.current_workflow_run_id, wr.status workflow_run_status, wr.progress,
        wr.executor_id
    from job.jobs j
    join workflow.workflows w
    on j.workflow_id = w.workflow_id
    left join workflow_run.workflow_runs wr
    on j.current_workflow_run_id = wr.workflow_run_id;

grant select on job.v_jobs to we_web;

comment on view job.v_jobs IS $$
Utility view, showing all jobs with workflow and possible workflow run (if currently running)
details.
$$;
