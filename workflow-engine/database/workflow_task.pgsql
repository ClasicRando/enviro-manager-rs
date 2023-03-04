create type workflow_engine.workflow_task as
(
    task_order integer,
    task_id bigint,
    name text,
    description text,
    parameters jsonb,
    service_name text,
    url text
);

comment on type workflow_engine.workflow_task IS $$
Container for information about a workflow task. Joins data from workflow_tasks, tasks and
task_services.
$$;
