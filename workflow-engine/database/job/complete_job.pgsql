create or replace procedure job.complete_job(
    job_id bigint,
    is_complete boolean
)
security definer
language sql
as $$
update job.jobs j
set
    current_workflow_run_id = case when $2 then null else j.current_workflow_run_id end,
    is_paused = not $2
where j.job_id = $1
$$;

grant execute on procedure job.complete_job to we_web;

comment on procedure job.complete_job IS $$
Sets a specified job to complete if a failure did not occur (denoted by the is_complete parameter)

Arguments:
job_id:
    ID of the job to run
is_complete:
    Flag indicating if the job was completed without errors. If false, the job is set to pause
$$;
