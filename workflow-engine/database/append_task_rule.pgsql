create or replace procedure workflow.append_task_rule(
    workflow_run_id bigint,
    task_order integer,
    rule task.task_rule
)
language sql
as $$
update task.task_queue tq
set
    rules = coalesce(rules,'{}'::task.task_rule[]) || $3
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::task.task_status;
$$;

comment on procedure task.append_task_rule IS $$
Add a new task rule to a task queue record.

Arguments:
workflow_run_id:
    ID of the workflow to schedule for running
task_order:
    Task order within the workflow run to be updated
rule:
    New task rule to add to rules array
$$;
