create type job.schedule_entry as
(
    day_of_week smallint,
    time_of_day time without time zone
);

grant usage on type job.schedule_entry to we_web;

comment on type job.schedule_entry IS $$
Container for information about the schedule of a job. Tells the system at what points in the
week the job needs to be run. For day_of_week, Monday is 1
$$;
