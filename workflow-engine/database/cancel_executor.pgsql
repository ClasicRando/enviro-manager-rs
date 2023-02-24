create or replace procedure workflow_engine.cancel_executor(
    executor_id bigint
)
language sql
as $$
update workflow_engine.registered_we_executors e
set status = 'Canceled'::workflow_engine.executor_status
where
    e.executor_id = $1
    and e.status = 'Active'::workflow_engine.executor_status;
$$;

comment on procedure workflow_engine.cancel_executor IS $$
Manually set an executor to cancel operations. This will send a message to the executor to start a
forced shutdown.

Arguments:
executor_id:
    ID of the executor to be manually forcefully shut down
$$;
