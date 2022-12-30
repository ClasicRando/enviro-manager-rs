create view workflow_engine.v_jobs as
    select j.job_id, j.workflow_id, w.name workflow_name, j.job_type,
           j.maintainer, j.job_schedule, j.job_interval, j.is_paused, j.next_run,
           j.current_workflow_run_id, wr.status workflow_run_status, wr.progress,
           wr.executor_id
    from   workflow_engine.jobs j
    join   workflow_engine.workflows w on j.workflow_id = w.workflow_id
    left join workflow_engine.workflow_runs wr on j.current_workflow_run_id = wr.workflow_run_id;

comment on view workflow_engine.v_jobs IS $$
Utility view, showing all jobs with workflow and possible workflow run (if currently running)
details.
$$;
