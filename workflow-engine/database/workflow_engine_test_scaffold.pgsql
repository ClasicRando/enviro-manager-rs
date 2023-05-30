declare
    v_task_service_id bigint;
    v_task_id bigint;
    v_workflow_id bigint;
    v_executor_id bigint;
    v_workflow_run_id bigint;
begin
    truncate
        task.task_services,
        task.tasks,
        workflow.workflows,
        task.workflow_tasks,
        executor.executors,
        workflow.workflow_runs,
        task.task_queue
        restart identity
        cascade;

    insert into task.task_services as t(name, base_url)
    values('test','http://127.0.0.1')
    returning t.service_id into v_task_service_id;

    insert into task.tasks as t(name, description, task_service_id, url)
    values('test','test data task insert', v_task_service_id, 'test')
    returning t.task_id into v_task_id;

    insert into workflow.workflows as w(name)
    values('test')
    returning w.workflow_id into v_workflow_id;

    insert into task.workflow_tasks(workflow_id, task_order, task_id, parameters)
    values(v_workflow_id, 1, v_task_id, null);

    insert into executor.executors as e(pid, username, application_name, client_addr, client_port)
    values(0, 'test', 'test_data', '127.0.0.1'::inet, 0)
    returning e.executor_id into v_executor_id;

    insert into workflow.workflow_runs as wr(workflow_id, executor_id)
    values(v_workflow_id, v_executor_id)
    returning wr.workflow_run_id into v_workflow_run_id;

    execute format(
        'create table if not exists "task".%I partition of task.task_queue for values in (%L)',
        'task_queue_'||v_workflow_run_id,
        v_workflow_run_id
    );

    insert into task.task_queue as tq(workflow_run_id, task_order, task_id, parameters, status)
    select
        v_workflow_run_id workflow_run_id, wt.task_order, wt.task_id,
        wt.parameters, 'Running'::task.task_status status
    from task.workflow_tasks wt
    join task.tasks t on wt.task_id = t.task_id
    where wt.workflow_id = v_workflow_id;
end;