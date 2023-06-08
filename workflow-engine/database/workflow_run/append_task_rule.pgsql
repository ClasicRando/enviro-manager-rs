create or replace procedure workflow_run.append_task_rule(
    workflow_run_id bigint,
    task_order integer,
    rule workflow_run.task_rule
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set
    rules = coalesce(rules,'{}'::workflow_run.task_rule[]) || $3
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::workflow_run.task_status;
$$;

grant execute on procedure workflow_run.append_task_rule to we_web;

comment on procedure workflow_run.append_task_rule IS $$
Add a new task rule to a task queue record.

Arguments:
workflow_run_id:
    ID of the workflow to schedule for running
task_order:
    Task order within the workflow run to be updated
rule:
    New task rule to add to rules array
$$;
