if not data_check.type_exists('workflow_engine','executor_status') then
    create type workflow_engine.executor_status as enum (
        'Active',
        'Canceled',
        'Shutdown'
    );
end if;

comment on type workflow_engine.executor_status IS $$
Various states that dictate the lifecycle of an executor. 'Active' is the default state when
initialized. 'Canceled' is a forced shutdown with no recovering of work in progress. 'Shutdown'
is a graceful shutdown where work in progress is finished until any active workflow runs can be
transferred to another worker (or paused for future continutation).
$$;
