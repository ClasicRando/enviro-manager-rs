create or replace view workflow.v_workflows as
with w_tasks as (
    select
        wt.workflow_id,
        array_agg(
            row(
                wt.task_order,
                t.task_id,
                t.name,
                t.description,
                wt.parameters,
                t.task_service_name,
                t.url
            )::workflow.workflow_task
        ) tasks
    from workflow.workflow_tasks wt
    join workflow.v_tasks t
    on wt.task_id = t.task_id
    group by wt.workflow_id
)
select w.workflow_id, w.name, wt.tasks
from workflow.workflows w
join w_tasks wt
on w.workflow_id = wt.workflow_id;

grant select on workflow.v_workflows to we_web;

comment on view workflow.v_workflows IS $$
Utility view, showing all workflows and their task details (as an array of workflow_task instances)
$$;
