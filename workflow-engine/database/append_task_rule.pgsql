create procedure workflow_engine.append_task_rule(
    workflow_run_id bigint,
    task_order integer,
    rule workflow_engine.task_rule
)
language sql
as $$
update workflow_engine.task_queue
set    rules = coalesce(rules,'{}'::workflow_engine.task_rule[]) || $3
where  workflow_run_id = $1
and    task_order = $2
and    status = 'Running'::workflow_engine.task_status;
$$;

comment on procedure workflow_engine.append_task_rule IS $$
Add a new task rule to a task queue record.

Arguments:
workflow_run_id:    ID of the workflow to schedule for running
task_order:         Task order within the workflow run to be updated
rule:               New task rule to add to rules array
$$;
