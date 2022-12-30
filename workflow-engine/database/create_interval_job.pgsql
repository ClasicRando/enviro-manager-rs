create function workflow_engine.create_interval_job(
    workflow_id bigint,
    maintainer text,
    job_interval interval,
    next_run timestamp without time zone default null
) returns bigint
language sql
as $$
insert into workflow_engine.jobs(workflow_id,job_type,maintainer,job_interval,next_run)
values($1,'Interval'::workflow_engine.job_type,$2,$3,coalesce($4, now() at time zone 'UTC' + $3))
returning job_id;
$$;

comment on function workflow_engine.create_interval_job IS $$
Create a new interval based job

Arguments:
workflow_id:    ID of the workflow as a template for the new job
maintainer:     Email of the maintainer of the job
job_interval:   Positive interval to dictate how frequent this job is run
next_run:       Optional parameter to decide when the job first runs. Default is to run 1
                job_interval from the current timestamp
$$;
