create type job.job_type as enum (
    'Scheduled',
    'Interval'
);

grant usage on type job.job_type to we_web;

comment on type job.job_type IS $$
Types of jobs, made distinct by the execution interval/schedule.
$$;
