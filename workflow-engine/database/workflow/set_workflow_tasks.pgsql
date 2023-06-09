create or replace procedure workflow.set_workflow_tasks(
    workflow_id bigint,
    tasks workflow.workflow_task_request[]
)
security definer
language sql
as $$
insert into workflow.workflow_tasks(workflow_id, task_order, task_id, parameters)
select w.workflow_id, row_number() over (), t.task_id, t.parameters
from workflow.workflows w
cross join unnest($2) t
where w.workflow_id = $1
on conflict(workflow_id, task_order) do
update set
    task_id = excluded.task_id,
    parameters = excluded.parameters
$$;

grant execute on procedure workflow.set_workflow_tasks to we_web;

comment on procedure workflow.set_workflow_tasks IS $$
Set the tasks of a workflow as a new set of task request entries.

Arguments:
tasks:
    Tasks to include in the workflow. Only includes task_id and optional parameters. Order in the
    array are taken as the order used during a workflow run.
$$;
