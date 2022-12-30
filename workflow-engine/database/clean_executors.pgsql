create or replace procedure workflow_engine.clean_executors()
language sql
as $$
with executors as (
    update workflow_engine.registered_we_executors
    set    status = 'Canceled'::workflow_engine.executor_status,
           exec_end = now() at time zone 'utc'
    where  status = 'Active'::workflow_engine.executor_status
    and    pid not in (select pid from pg_stat_activity)
    returning executor_id
), workflows as (
    update workflow_engine.workflow_runs wr
    set    status = 'Canceled'::workflow_engine.workflow_run_status,
           executor_id = null
    from   executors e
    where  wr.executor_id = e.executor_id
    returning workflow_run_id
)
update workflow_engine.task_queue tq
set    status = 'Canceled'::workflow_engine.task_status
from   workflows w
where  tq.workflow_run_id = w.workflow_run_id
and    tq.status = 'Running'::workflow_engine.task_status;
$$;

comment on procedure workflow_engine.clean_executors IS $$
Cleans any executors that are no longer attached to the database but have not been shutdown
correctly. Also cleans all the workflows and task queue entires attached to the invalid
executors.
$$;
