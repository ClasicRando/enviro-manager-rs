create or replace view workflow_run.v_workflow_runs as
with tasks as (
    select
        tq.workflow_run_id,
        array_agg(
            row(
                tq.task_order,
                tq.task_id,
                t.name,
                t.description,
                tq.status,
                tq.parameters,
                tq.output,
                tq.rules,
                tq.task_start,
                tq.task_end,
                tq.progress
            )::workflow_run.workflow_run_task
            order by tq.task_id
        ) as tasks
    from workflow_run.task_queue tq
    join workflow.tasks t on t.task_id = tq.task_id
    group by tq.workflow_run_id
)
select wr.workflow_run_id, wr.workflow_id, wr.status, wr.executor_id, wr.progress, t.tasks
from workflow_run.workflow_runs wr
join tasks t on wr.workflow_run_id = t.workflow_run_id;

grant select on workflow_run.v_workflow_runs to we_web;

comment on view workflow_run.v_workflow_runs IS $$
Utility view, showing workflow runs with details about the workflows as needed
$$;
