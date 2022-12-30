if not data_check.type_exists('workflow_engine','workflow_run_status') then
    create type workflow_engine.workflow_run_status as enum (
        'Waiting',
        'Scheduled',
        'Running',
        'Paused',
        'Failed',
        'Complete',
        'Canceled'
    );
end if;

comment on type workflow_engine.workflow_run_status IS $$
Various states that dictate the lifecycle of a workflow. Starts with 'Waiting', 'Scheduled' when
ready to execute, 'Running' when picked up by an executor then 'Paused'/'Failed'/'Complete'/
'Canceled' when execution is done. 'Paused'/'Failed'/'Canceled' need to be reworked to be
successfully completed. 
$$;
