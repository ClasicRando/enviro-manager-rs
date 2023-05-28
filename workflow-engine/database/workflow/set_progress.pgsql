create or replace procedure workflow.set_progress(
    workflow_run_id bigint
)
security definer
language sql
as $$
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
    status = 'Scheduled'::workflow.workflow_run_status,
    progress = round((t.complete_count / cast(t.total_tasks as real)) * 100)::smallint
from tasks t
where
    wr.workflow_run_id = t.workflow_run_id
    and wr.workflow_run_id = $1
$$;

grant execute on procedure workflow.set_progress to we_web;

comment on procedure workflow.set_progress IS $$
Manually complete a task that is paused to continue with workflow run.

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task to complete
task_order:
    Task order within the workflow run to complete
$$;
