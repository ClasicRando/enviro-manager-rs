create or replace function job.next_run_job_schedule(
    job_schedule job.schedule_entry[]
) returns timestamp without time zone
immutable
returns null on null input
language sql
as $$
select
    case
        when is_next_week then start_of_next_week
        else start_of_week
    end  + time_since_start_of_week next_run
from (
    select
        time_since_start_of_week, start_of_week, start_of_next_week,
        (start_of_week + time_since_start_of_week) - current_dt < interval '0 second' is_next_week
    from (
        select
            make_interval(
                days => day_of_week::int - 1,
                hours => extract(hour from time_of_day)::int,
                mins => extract(minute from time_of_day)::int
            ) time_since_start_of_week,
            now() at time zone 'UTC' current_dt,
            date_trunc('week', now() at time zone 'UTC') start_of_week,
            date_trunc('week', now() at time zone 'UTC' + interval '7 days') start_of_next_week
        from unnest($1)
    ) t1
) t1
order by next_run
limit 1;
$$;

comment on function job.next_run_job_schedule IS $$
Returns the next schedule entry to run (for a 'Schedule' type job) as a future timestamp.

Arguments:
job_schedule:
    Array of job schedule entries to find the next schedule entry
$$;
