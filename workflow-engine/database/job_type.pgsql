create type workflow_engine.job_type as enum(
    'Scheduled',
    'Interval'
);

comment on type workflow_engine.job_type IS $$
Types of jobs, made distinct by the execution interval/schedule.
$$;
