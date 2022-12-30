create type workflow_engine.workflow_task_request as
(
    task_id bigint,
    parameters jsonb
);

comment on type workflow_engine.workflow_task_request IS $$
Container for information used to generate new tasks part of a new workflow
$$;
