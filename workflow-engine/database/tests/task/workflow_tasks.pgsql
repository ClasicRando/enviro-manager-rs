declare
	v_workflow_name text := 'workflow_task Trigger Test';
    v_tasks task.workflow_task_request[] := array[
        row(1,null)::task.workflow_task_request,
        row(1,null)::task.workflow_task_request
    ];
    v_workflow_id bigint;
	v_unexpected_exit boolean;
    v_state text;
    v_msg text;
    v_detail text;
    v_hint text;
    v_context text;
begin
	delete from task.workflow_tasks wt
	using workflow.workflows w
	where
		wt.workflow_id = w.workflow_id
		and w.name = v_workflow_name;
	
	delete from workflow.workflows w
	where w.name = v_workflow_name;
	
    v_workflow_id := workflow.create_workflow(v_workflow_name, v_tasks);
	v_unexpected_exit := true;
    begin
        insert into task.workflow_tasks(workflow_id, task_order, task_id)
        values(v_workflow_id, 3, 1),(1, 2, 1);
    exception
        when others then
            get stacked diagnostics
                v_msg     = message_text;
			assert
				v_msg = 'When affecting workflow_tasks, only 1 workflow_id can be impacted',
				format(
					'An exception must be raised when trying to insert workflow tasks of multiple workflows. Error %s',
					v_msg
				);
			v_unexpected_exit := false;
    end;
	
	if v_unexpected_exit then
		raise exception 'Unexpected exit of block without capturing an exception';
	end if;

	v_unexpected_exit := true;
    begin
        update task.workflow_tasks
        set parameters = '{"test":null}'::jsonb;
    exception
        when others then
            get stacked diagnostics
                v_msg     = message_text;
			assert
				v_msg = 'When affecting workflow_tasks, only 1 workflow_id can be impacted',
				format(
					'An exception must be raised when trying to insert workflow tasks of multiple workflows. Error %s',
					v_msg
				);
			v_unexpected_exit := false;
    end;
	
	if v_unexpected_exit then
		raise exception 'Unexpected exit of block without capturing an exception';
	end if;

	v_unexpected_exit := true;
    begin
        delete from task.workflow_tasks;
    exception
        when others then
            get stacked diagnostics
                v_msg     = message_text;
			assert
				v_msg = 'When affecting workflow_tasks, only 1 workflow_id can be impacted',
				format(
					'An exception must be raised when trying to insert workflow tasks of multiple workflows. Error %s',
					v_msg
				);
			v_unexpected_exit := false;
    end;
	
	if v_unexpected_exit then
		raise exception 'Unexpected exit of block without capturing an exception';
	end if;

	v_unexpected_exit := true;
    begin
        update task.workflow_tasks wt
        set task_order = 3
        where
            wt.workflow_id = v_workflow_id
            and wt.task_order = 2;
    exception
        when others then
            get stacked diagnostics
                v_msg     = message_text;
			assert
				v_msg like 'Task order values are not correct. See these instances:%',
				format(
					'An exception must be raised when trying to insert workflow tasks of multiple workflows. Error %s',
					v_msg
				);
			v_unexpected_exit := false;
    end;
	
	if v_unexpected_exit then
		raise exception 'Unexpected exit of block without capturing an exception';
	end if;
end;
