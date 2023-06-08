create type workflow_run.workflow_run_status as enum (
    'Waiting',
    'Scheduled',
    'Running',
    'Paused',
    'Failed',
    'Complete',
    'Canceled'
);

comment on type workflow_run.workflow_run_status IS $$
Various states that dictate the lifecycle of a workflow_run. Starts with 'Waiting', 'Scheduled' when
ready to execute, 'Running' when picked up by an executor then 'Paused'/'Failed'/'Complete'/
'Canceled' when execution is done. 'Paused'/'Failed'/'Canceled' need to be reworked to be
successfully completed. 
$$;
