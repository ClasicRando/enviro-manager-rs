declare
    v_executor_id bigint := executor.register_executor();
    v_status executor.executor_status;
begin
    call executor.shutdown_executor(v_executor_id);

    select status
    into v_status
    from executor.executors
    where executor_id = v_executor_id;

    assert
        v_status = 'Shutdown'::executor.executor_status,
        'Executor status did not set the status as "Shutdown"';
end;