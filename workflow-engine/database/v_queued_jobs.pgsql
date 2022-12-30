create view workflow_engine.v_queued_jobs
    select job_id, next_run
    from   workflow_engine.jobs j
    where  not is_paused
    and    not exists(
        select 1
        from   workflow_engine.workflow_runs wr
        where  j.current_workflow_run_id = wr.workflow_run_id
        and    wr.status != 'Complete'
    )
    order by next_run;

comment on view workflow_engine.v_queued_jobs IS $$
Get all jobs that are not paused and is not currently running
$$;
