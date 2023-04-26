declare
    v_task_service_id bigint;
    v_task_id bigint;
    v_workflow_id bigint;
    v_executor_id bigint;
    v_workflow_run_id bigint;
begin
    begin
        select t.service_id
        into strict v_task_service_id
        from task.task_services t
        where t.name = 'test';
    exception
        when no_data_found then
            insert into task.task_services as t (name, base_url)
            values('test', 'http:\\test')
            returning t.service_id into v_task_service_id;
    end;

    begin
        select t.task_id
        into strict v_task_id
        from task.tasks t
        where t.name = 'test';
    exception
        when no_data_found then
            insert into task.tasks as t (name, description, task_service_id, url)
            values('test','test data task insert', v_task_service_id, 'test')
            returning t.task_id into v_task_id;
    end;

    merge into workflow.workflows w
    using (values('test')) v(name)
    on (w.name = v.name)
    when matched then
    update set
        is_deprecated = false,
        new_workflow = null
    when not matched then
    insert(name)
    values(v.name);

    select w.workflow_id
    into strict v_workflow_id
    from workflow.workflows w
    where w.name = 'test';

    if not exists (
        select 1
        from task.workflow_tasks wt
        where wt.workflow_id = v_workflow_id
        and wt.task_order = 1
    ) then
        insert into task.workflow_tasks(workflow_id, task_order, task_id, parameters)
        values(v_workflow_id, 1, v_task_id, null);
    end if;
    

    merge into executor.executors as e
    using (
        select 'test' username, 'test_data' application_name, '127.0.0.1'::inet client_addr
    ) as t
    on (
        e.username = t.username
        and e.application_name = t.application_name
        and e.client_addr = t.client_addr
    )
    when matched then
    update set
        status = 'Active'::executor.executor_status
    when not matched then
    insert(pid, username, application_name, client_addr, client_port)
    values(0, t.username, t.application_name, t.client_addr, 0);
    
    select e.executor_id
    into v_executor_id
    from executor.executors e
    where
        e.username = 'test'
        and e.application_name = 'test_data'
        and e.client_addr = '127.0.0.1'::inet;

    merge into workflow.workflow_runs as wr
    using (select v_workflow_id workflow_id, v_executor_id executor_id) as t
    on (wr.workflow_id = t.workflow_id)
    when matched then
    update set
        executor_id = t.executor_id,
        status = 'Waiting'::workflow.workflow_run_status
    when not matched then
    insert(workflow_id, executor_id)
    values(t.workflow_id, t.executor_id);
    
    select wr.workflow_run_id
    into v_workflow_run_id
    from workflow.workflow_runs wr
    where wr.workflow_id = v_workflow_id;

    execute format(
        'create table if not exists "task".%I partition of task.task_queue for values in (%L)',
        'task_queue_'||v_workflow_run_id,
        v_workflow_run_id
    );

    merge into task.task_queue as tq
    using (
        select
            v_workflow_run_id workflow_run_id, wt.task_order, wt.task_id,
            wt.parameters, 'Running'::task.task_status status
        from task.workflow_tasks wt
        join task.tasks t on wt.task_id = t.task_id
        where wt.workflow_id = v_workflow_id
    ) as t
    on (
        tq.workflow_run_id = t.workflow_run_id
        and tq.task_order = t.task_order
    )
    when matched then
    update set
        status = t.status
    when not matched then
    insert(workflow_run_id, task_order, task_id, parameters, status)
    values(t.workflow_run_id, t.task_order, t.task_id, t.parameters, t.status);

end;