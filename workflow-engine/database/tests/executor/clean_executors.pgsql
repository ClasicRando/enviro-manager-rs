declare
    r_executor record;
begin
    create temporary table executors_before on commit drop as
    select
        e.executor_id,
        e.pid in (select pid from pg_stat_activity) is_alive,
        count(distinct wr.workflow_run_id) wr_count,
        count(0) filter (where tq.status = 'Canceled'::task.task_status) cancel_status,
        count(0) filter (where tq.status = 'Running'::task.task_status) running_status
    from executor.executors e
    join workflow.workflow_runs wr on e.executor_id = wr.executor_id
    join task.task_queue tq on wr.workflow_run_id = tq.workflow_run_id
    where e.status = 'Active'::executor.executor_status
    group by e.executor_id, e.pid;

    call executor.clean_executors();

    create temporary table executors_after on commit drop as
    select
        e.executor_id,
        e.pid in (select pid from pg_stat_activity) is_alive,
        count(distinct wr.workflow_run_id)
            filter (where wr.status = 'Canceled'::workflow.workflow_run_status) wr_cancelled_count,
        count(0) filter (where tq.status = 'Canceled'::task.task_status) cancel_status
    from executor.executors e
    join workflow.workflow_runs wr on e.executor_id = wr.executor_id
    join task.task_queue tq on wr.workflow_run_id = tq.workflow_run_id
    where e.status = 'Active'::executor.executor_status
    group by e.executor_id, e.pid;

    for r_executor in (
        select
            coalesce(eb.executor_id, ea.executor_id) executor_id,
            eb.is_alive was_alive, ea.is_alive,
            eb.wr_count wr_count_before, ea.wr_cancelled_count wr_cancelled_after,
            eb.cancel_status + eb.running_status to_be_canceled_before,
            ea.cancel_status cancel_status_after
        from executors_before eb
        full join executors_after ea on eb.executor_id = ea.executor_id
    )
    loop
        if not r_executor.was_alive then
            continue; 
        end if;

        assert
            not r_executor.is_alive,
            'Executor = % is still alive after cleanup',
            r_executor.executor_id;

        assert
            r_executor.wr_count_before = r_executor.wr_cancelled_after,
            'Not all workflow runs for Executor = % were canceled',
            r_executor.executor_id;

        assert
            r_executor.to_be_canceled_before = r_executor.cancel_status_after,
            'Not all task queue records for Executor = % were canceled',
            r_executor.executor_id;
    end loop;
end;