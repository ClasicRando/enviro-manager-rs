create type workflow_engine.schedule_entry as
(
    day_of_week smallint,
    time_of_day time without time zone
);

comment on type workflow_engine.schedule_entry IS $$
Container for information about the schedule of a job. Tells the system at what points in the
week the job needs to be run. For day_of_week, Monday is 1
$$;
