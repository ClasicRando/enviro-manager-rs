create or replace procedure executor.shutdown_executor(
    executor_id bigint
)
language sql
as $$
update executor.executors e
set status = 'Shutdown'::executor.executor_status
where
    e.executor_id = $1
    and e.status = 'Active'::executor.executor_status;
$$;

comment on procedure executor.shutdown_executor IS $$
Set the status of an executor to 'Shutdown' which sends a notification to the executor to perform
a graceful shutdown.

Arguments:
executor_id:
    ID of the executor that will be shut down
$$;
