create type workflow.workflow_task_request as
(
    task_id bigint,
    parameters jsonb
);

grant usage on type workflow.workflow_task_request to we_web;

comment on type workflow.workflow_task_request IS $$
Container for information used to generate new tasks part of a new workflow
$$;
