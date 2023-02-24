create or replace procedure workflow_engine.shutdown_executor(
    executor_id bigint
)
language sql
as $$
update workflow_engine.registered_we_executors e
set status = 'Shutdown'::workflow_engine.executor_status
where
    e.executor_id = $1
    and e.status = 'Active'::workflow_engine.executor_status;
$$;

comment on procedure workflow_engine.shutdown_executor IS $$
Set the status of an executor to 'Shutdown' which sends a notification to the executor to perform
a graceful shutdown.

Arguments:
executor_id:
    ID of the executor that will be shut down
$$;
