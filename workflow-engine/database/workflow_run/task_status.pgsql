create type workflow_run.task_status as enum (
    'Waiting',
    'Running',
    'Paused',
    'Failed',
    'Rule Broken',
    'Complete',
    'Canceled'
);

grant usage on type workflow_run.task_status to we_web;

comment on type workflow_run.task_status IS $$
Various states that dictate the lifecycle of a workflow_run. Starts with 'Waiting', 'Running' when picked
up by an workflow run worker then 'Paused'/'Failed'/'Rule Broken'/'Complete'/'Canceled' when
execution is done. 'Paused'/'Failed'/'Rule Broken'/'Canceled' need to be restarted to be
successfully completed. 
$$;
