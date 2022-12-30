if not data_check.type_exists('workflow_engine','task_status') then
    create type workflow_engine.task_status as enum (
        'Waiting',
        'Running',
        'Paused',
        'Failed',
        'Rule Broken',
        'Complete',
        'Canceled'
    );
end if;

comment on type workflow_engine.task_status IS $$
Various states that dictate the lifecycle of a task. Starts with 'Waiting', 'Running' when picked
up by an workflow run worker then 'Paused'/'Failed'/'Rule Broken'/'Complete'/'Canceled' when
execution is done. 'Paused'/'Failed'/'Rule Broken'/'Canceled' need to be restarted to be
successfully completed. 
$$;
