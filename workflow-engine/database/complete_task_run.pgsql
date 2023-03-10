create or replace procedure workflow.complete_task_run(
    workflow_run_id bigint,
    task_order integer,
    is_paused boolean,
    output text
)
language sql
as $$
update task.task_queue tq
set   
    status = case
        when exists(select 1 from unnest(rules) where failed) then 'Rule Broken'::task.task_status
        when $3 then 'Paused'::task.task_status
        else 'Complete'::task.task_status
    end,
    output = $4,
    task_end = now() at time zone 'UTC',
    progress = 100
where
    tq.workflow_run_id = $1
    and tq.task_order = $2
    and tq.status = 'Running'::task.task_status;

with tasks as (
    select
        tq.workflow_run_id,
        count(0) filter (where tq.status = 'Complete'::task.task_status) complete_count,
        count(0) total_tasks
    from task.task_queue tq
    group by tq.workflow_run_id
)
update workflow.workflow_runs wr
set
    progress = round((t.complete_count / cast(t.total_tasks as real)) * 100)::smallint
from tasks t
where
    wr.workflow_run_id = t.workflow_run_id
    and wr.workflow_run_id = $1;
$$;

comment on procedure workflow.complete_task_run IS $$
Set the task record as done with either a 'Rule Broken', 'Paused' or 'Complete' status. Optional
message as output is also available

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task
task_order:
    Task order within the workflow run to be run
is_paused:
    Flag denoting if the result of the task instructs the workflow run to pause
output:
    Message output from the task run, can be null if no message is required
$$;
