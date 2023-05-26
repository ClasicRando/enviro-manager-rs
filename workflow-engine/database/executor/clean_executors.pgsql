create or replace procedure executor.clean_executors()
security definer
language sql
as $$
with executors as (
    update executor.executors e
    set
        status = 'Canceled'::executor.executor_status,
        exec_end = now() at time zone 'utc'
    where
        e.status = 'Active'::executor.executor_status
        and e.pid not in (select pid from pg_stat_activity)
    returning executor_id
), workflows as (
    update workflow.workflow_runs wr
    set
        status = 'Canceled'::workflow.workflow_run_status,
        executor_id = null
    from executors e
    where wr.executor_id = e.executor_id
    returning workflow_run_id
)
update task.task_queue tq
set status = 'Canceled'::task.task_status
from workflows w
where 
    tq.workflow_run_id = w.workflow_run_id
    and tq.status = 'Running'::task.task_status;
$$;

revoke all on procedure executor.clean_executors;
grant execute on procedure executor.clean_executors;

comment on procedure executor.clean_executors IS $$
Cleans any executors that are no longer attached to the database but have not been shutdown
correctly. Also cleans all the workflows and task queue entries attached to the invalid
executors.
$$;
