create type job.job_type as enum(
    'Scheduled',
    'Interval'
);

comment on type job.job_type IS $$
Types of jobs, made distinct by the execution interval/schedule.
$$;
