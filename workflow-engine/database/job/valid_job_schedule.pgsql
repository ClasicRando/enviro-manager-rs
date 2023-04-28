create or replace function job.valid_job_schedule(
    job_schedule job.schedule_entry[]
) returns boolean
immutable
language plpgsql
as $$
declare
    entry job.schedule_entry;
    v_duplicate_entry_count integer;
begin
    if job_schedule is null or job_schedule = '{}'::job.schedule_entry[] then
        return false;
    end if;

    select count(0)
    into v_duplicate_entry_count
    from (
        select 1
        from unnest($1) s
        group by s.day_of_week, s.time_of_day
        having count(0) > 1
    ) d;

    if v_duplicate_entry_count > 0 then
        return false;
    end if;

    foreach entry in array job_schedule
    loop
        if not entry.day_of_week between 1 and 7 then
            return false;
        end if;
    end loop;
    return true;
end;
$$;

comment on function job.valid_job_schedule IS $$
Validates the job schedule is not null, contains 1 or more entries and the entries are valid.
Valid entries are when the day_of_week attribute is between 1 and 7.

Arguments:
job_schedule:
    Array of job schedule entries to verify
$$;
