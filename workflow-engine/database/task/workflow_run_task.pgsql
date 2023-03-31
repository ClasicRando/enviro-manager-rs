create type task.workflow_run_task as
(
    task_order integer,
    task_id bigint,
    name text,
    description text,
    task_status task.task_status,
    parameters jsonb,
    output text,
    rules task.task_rule[],
    task_start timestamp without time zone,
    task_end timestamp without time zone,
    progress smallint
);

comment on type task.workflow_run_task IS $$
Container for information about a workflow run task. Joins data from workflow_runs and tasks
$$;