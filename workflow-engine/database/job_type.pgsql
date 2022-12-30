if not data_check.type_exists('workflow_engine','job_type') then
    create type workflow_engine.job_type as enum(
        'Scheduled',
        'Interval'
    );
end;

comment on type workflow_engine.job_type IS $$
Types of jobs, made distinct by the execution interval/schedule.
$$;
