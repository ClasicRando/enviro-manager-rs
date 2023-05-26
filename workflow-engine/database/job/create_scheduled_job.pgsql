create or replace function job.create_scheduled_job(
    workflow_id bigint,
    maintainer text,
    job_schedule job.schedule_entry[]
) returns bigint
security definer
language sql
as $$
insert into job.jobs(workflow_id,job_type,maintainer,job_schedule,next_run)
values($1,'Scheduled'::job.job_type,$2,$3,job.next_run_job_schedule($3))
returning job_id;
$$;

grant execute function on job.create_scheduled_job to we_web;

comment on function job.create_scheduled_job IS $$
Create a new weekly schedule based job 

Arguments:
workflow_id:
    ID of the workflow as a template for the new job
maintainer:
    Email of the maintainer of the job
job_schedule:
    Schedule of 1 or more weekly time slots to run the job
$$;
