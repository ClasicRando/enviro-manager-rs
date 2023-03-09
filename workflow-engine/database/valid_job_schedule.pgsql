create or replace function job.valid_job_schedule(
    job_schedule job.schedule_entry[]
) returns boolean
immutable
language plpgsql
as $$
declare
    entry job.schedule_entry;
begin
    if job_schedule is null or job_schedule = '{}'::job.schedule_entry[] then
        return false;
    end if;

    foreach entry in array job_schedule
    loop
        if not entry.day_of_week between 1 and 7 then
            return false;
        end if;
    end loop;
end;
$$;

comment on function job.valid_job_schedule IS $$
Validates the job schedule is not null, contains 1 or more entries and the entries are valid.
Valid entries are when the day_of_week attribute is between 1 and 7.

Arguments:
job_schedule:
    Array of job schedule entries to verify
$$;
