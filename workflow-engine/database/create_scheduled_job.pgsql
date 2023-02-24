create or replace function workflow_engine.create_scheduled_job(
    workflow_id bigint,
    maintainer text,
    job_schedule workflow_engine.schedule_entry[]
) returns bigint
language sql
as $$
insert into workflow_engine.jobs(workflow_id,job_type,maintainer,job_schedule,next_run)
values($1,'Interval'::workflow_engine.job_type,$2,$3,workflow_engine.next_run_job_schedule($3))
returning job_id;
$$;

comment on function workflow_engine.create_scheduled_job IS $$
Create a new weekly schedule based job 

Arguments:
workflow_id:
    ID of the workflow as a template for the new job
maintainer:
    Email of the maintainer of the job
job_schedule:
    Schedule of 1 or more weekly time slots to run the job
$$;
