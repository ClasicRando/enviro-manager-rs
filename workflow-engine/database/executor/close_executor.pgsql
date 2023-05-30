create or replace procedure executor.close_executor(
    executor_id bigint,
    is_cancelled boolean default false
)
security definer
language sql
as $$
update executor.executors e
set
    exec_end = now() at time zone 'UTC',
    status = case
        when $2 then 'Canceled'::executor.executor_status
        else 'Shutdown'::executor.executor_status
    end
where e.executor_id = $1;

with workflow_runs as (
    update workflow.workflow_runs wr
    set
        status = 'Canceled'::workflow.workflow_run_status,
        executor_id = null
    where
        wr.executor_id = $1
        and wr.status = 'Running'::workflow.workflow_run_status
    returning workflow_run_id
)
update task.task_queue tq
set
    status = 'Canceled'::task.task_status,
    task_end = now() at time zone 'UTC',
    output = 'Task executor canceled workflow run'
from workflow_runs wr
where
    wr.workflow_run_id = tq.workflow_run_id
    and tq.status = 'Running'::task.task_status;
$$;

grant execute on procedure executor.close_executor to we_web;

comment on procedure executor.close_executor IS $$
Close the operations of an executor, including child operations of a workflow run.

task_queue and workflow_runs are updated if the executor still owns any workflow runs.

Arguments:
executor_id:
    ID of the executor to close
is_cancelled:
    True if executor was a force shutdown and false if a graceful shutdown, default is false
$$;
