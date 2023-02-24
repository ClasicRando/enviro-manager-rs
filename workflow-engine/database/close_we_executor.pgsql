create or replace procedure workflow_engine.close_we_executor(
    executor_id bigint,
    is_cancelled boolean default false
)
language sql
as $$
update workflow_engine.registered_we_executors e
set
    exec_end = now() at time zone 'UTC',
    status = case
        when $2 then 'Canceled'::workflow_engine.executor_status
        else 'Shutdown'::workflow_engine.executor_status
    end
where e.executor_id = $1;

with workflow_runs as (
    update workflow_engine.workflow_runs wr
    set
        status = 'Canceled'::workflow_engine.workflow_run_status,
        executor_id = null
    where
        wr.executor_id = $1
        and wr.status = 'Running'::workflow_engine.workflow_run_status
    returning workflow_run_id
)
update workflow_engine.task_queue tq
set
    status = 'Canceled'::workflow_engine.task_status,
    task_end = now() at time zone 'UTC',
    output = 'Task executor canceled workflow run'
from workflow_runs wr
where
    wr.workflow_run_id = tq.workflow_run_id
    and tq.status = 'Running'::workflow_engine.task_status;
$$;

comment on procedure workflow_engine.close_we_executor IS $$
Close the operations of an executor, including child operations of a workflow run.

task_queue and workflow_runs are updated if the executor still owns any workflow runs.

Arguments:
executor_id:
    ID of the exectuor to close
is_cancelled:
    True if executor was a force shutdown and false if a graceful shutdown, default is false
$$;
