create or replace view workflow_engine.v_queued_jobs as
select j.job_id, j.next_run
from workflow_engine.jobs j
where 
    not j.is_paused
    and not exists(
        select 1
        from workflow_engine.workflow_runs wr
        where j.current_workflow_run_id = wr.workflow_run_id
        and wr.status != 'Complete'
    )
order by j.next_run;

comment on view workflow_engine.v_queued_jobs IS $$
Get all jobs that are not paused and is not currently running
$$;
