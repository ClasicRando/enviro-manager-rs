create or replace function workflow.create_workflow(
    name text,
    tasks task.workflow_task_request[]
) returns bigint
language sql
as $$
with workflow as (
    insert into workflow.workflows(name)
    values($1)
    returning workflow_id
), workflow_tasks as (
    insert into task.workflow_tasks(workflow_id, task_order, task_id, parameters)
    select w.workflow_id, row_number() over (), t.task_id, t.parameters
    from workflow w
    cross join unnest($2) t
    returning workflow_id
)
select distinct wt.workflow_id
from workflow_tasks wt
$$;

comment on function workflow.create_workflow IS $$
Create a new template workflow, aliased as the provided name and including the tasks provided.
Returns the new workflow id.

Arguments:
name:
    Alias given to the new workflow
tasks:
    Tasks included in the new workflow. Only includes task_id and optional parameters. Order
    in the array are taken as the order used during a workflow run.
$$;