create or replace procedure workflow_engine.complete_workflow_run(
    workflow_run_id bigint
)
language sql
as $$
with task_queue_status as (
    select
        tq.workflow_run_id,
        count(0) total_count,
        count(0) filter (where tq.status = 'Complete'::workflow_engine.task_status) complete_count,
        count(0) filter (where tq.status = 'Failed'::workflow_engine.task_status) failed_count,
        count(0) filter (where tq.status = 'Rule Broken'::workflow_engine.task_status) rule_broke_count,
        count(0) filter (where tq.status = 'Paused'::workflow_engine.task_status) paused_count,
        count(0) filter (where tq.status = 'Canceled'::workflow_engine.task_status) canceled_count
    from workflow_engine.task_queue tq.
    where workflow_run_id = $1
    group by tq.workflow_run_id
)
update workflow_engine.workflow_runs wr
set
    status = case
        when tqs.total_count = tqs.complete_count then 'Complete'::workflow_engine.workflow_run_status
        when tqs.failed_count > 0 then 'Failed'::workflow_engine.workflow_run_status
        when tqs.rule_broke_count > 0 then 'Paused'::workflow_engine.workflow_run_status
        when tqs.paused_count > 0 then 'Paused'::workflow_engine.workflow_run_status
        when tqs.canceled_count > 0 then 'Canceled'::workflow_engine.workflow_run_status
        else 'Paused'::workflow_engine.workflow_run_status
    end,
    executor_id = null,
    progress = case
        when tqs.total_count = tqs.complete_count then 100
        when tqs.failed_count > 0 then null
        when tqs.rule_broke_count > 0 then 100
        when tqs.paused_count > 0 then 100
        when tqs.canceled_count > 0 then null
        else null
    end
from task_queue_status tqs
where wr.workflow_run_id = tqs.workflow_run_id;
$$;

comment on procedure workflow_engine.complete_workflow_run IS $$
Finish a workflow run, checking the task queue to assign a status.

Status is set using logical cascading:
    - if all tasks are completed successfully, status is 'Complete'
    - if 1 or more tasks failed, status is 'Failed'
    - if 1 or more tasks have broken rules, status is 'Rule Broken'
    - if 1 or more tasks are paused, status is 'Paused'
    - if 1 or more tasks are canceled, status is 'Canceled'
    - otherwise, the status is 'Paused' since the outcome is undefined

Arguments:
workflow_run_id:
    ID of the workflow run
$$;
