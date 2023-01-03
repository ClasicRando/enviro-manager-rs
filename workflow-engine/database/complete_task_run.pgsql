create or replace procedure workflow_engine.complete_task_run(
    workflow_run_id bigint,
    task_order integer,
    is_paused boolean,
    output text
)
language sql
as $$
update workflow_engine.task_queue
set    status = case
                    when exists(select 1 from unnest(rules) where failed) then 'Rule Broken'::workflow_engine.task_status
                    when $3 then 'Paused'::workflow_engine.task_status
                    else 'Complete'::workflow_engine.task_status
                end,
       output = $4,
       task_end = now() at time zone 'UTC',
       progress = 100
where  workflow_run_id = $1
and    task_order = $2
and    status = 'Running'::workflow_engine.task_status;

with tasks as (
    select workflow_run_id,
           count(0) filter (where status = 'Complete'::workflow_engine.task_status) complete_count,
           count(0) total_tasks
    from   workflow_engine.task_queue
    group by workflow_run_id
)
update workflow_engine.workflow_runs wr
set    progress = round((t.complete_count / cast(t.total_tasks as real)) * 100)::smallint
from   tasks t
where  wr.workflow_run_id = t.workflow_run_id
and    wr.workflow_run_id = $1;
$$;

comment on procedure workflow_engine.complete_task_run IS $$
Set the task record as done with either a 'Rule Broken', 'Paused' or 'Complete' status. Optional
message as output is also available

Arguments:
workflow_run_id:    ID of the workflow run that owns the task
task_order:         Task order within the workflow run to be run
is_paused:          Flag denoting if the result of the task instructs the workflow run to pause
output:             Message output from the task run, can be null if no message is required
$$;
