create type task.task_status as enum (
    'Waiting',
    'Running',
    'Paused',
    'Failed',
    'Rule Broken',
    'Complete',
    'Canceled'
);

comment on type task.task_status IS $$
Various states that dictate the lifecycle of a task. Starts with 'Waiting', 'Running' when picked
up by an workflow run worker then 'Paused'/'Failed'/'Rule Broken'/'Complete'/'Canceled' when
execution is done. 'Paused'/'Failed'/'Rule Broken'/'Canceled' need to be restarted to be
successfully completed. 
$$;
