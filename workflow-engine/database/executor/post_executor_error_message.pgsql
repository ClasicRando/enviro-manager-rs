create or replace procedure executor.post_executor_error_message(
    executor_id bigint,
    error_message text
)
language sql
as $$
update executor.executors
set error_message = $2
where executor_id = $1;
$$;

comment on procedure executor.post_executor_error_message IS $$
Post an error message for an executor to indicate the reason for it crashing

Arguments:
executor_id:
    ID of the executor that encountered an error
error_message:
    Message to post as the error reason for the executor crashing
$$;