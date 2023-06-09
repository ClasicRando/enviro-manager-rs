create type workflow_run.workflow_run_task as
(
    task_order integer,
    task_id bigint,
    name text,
    description text,
    task_status workflow_run.task_status,
    parameters jsonb,
    output text,
    rules workflow_run.task_rule[],
    task_start timestamp without time zone,
    task_end timestamp without time zone,
    progress smallint
);

grant usage on type workflow_run.workflow_run_task to we_web;

comment on type workflow_run.workflow_run_task IS $$
Container for information about a workflow run workflow_run. Joins data from workflow_runs and tasks
$$;
