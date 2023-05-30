create or replace procedure executor.cancel_executor(
    executor_id bigint
)
security definer
language sql
as $$
update executor.executors e
set status = 'Canceled'::executor.executor_status
where
    e.executor_id = $1
    and e.status = 'Active'::executor.executor_status;
$$;

grant execute on procedure executor.cancel_executor to we_web;

comment on procedure executor.cancel_executor IS $$
Manually set an executor to cancel operations. This will send a message to the executor to start a
forced shutdown.

Arguments:
executor_id:
    ID of the executor to be manually forcefully shut down
$$;
