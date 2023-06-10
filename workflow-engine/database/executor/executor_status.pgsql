create type executor.executor_status as enum (
    'Active',
    'Canceled',
    'Shutdown'
);

grant usage on type executor.executor_status to we_web;

comment on type executor.executor_status IS $$
Various states that dictate the lifecycle of an executor. 'Active' is the default state when
initialized. 'Canceled' is a forced shutdown with no recovering of work in progress. 'Shutdown'
is a graceful shutdown where work in progress is finished until any active workflow runs can be
transferred to another worker (or paused for future continuation).
$$;
