create or replace view workflow_run.v_task_queue_record as
    select tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, t.url
    from workflow_run.task_queue tq
    join workflow.v_tasks t
    on t.task_id = tq.task_id;

grant select on workflow_run.v_task_queue_record to we_web;

comment on view workflow_run.v_task_queue_record IS $$
Utility view, allows accessing task_queue as read-only.
$$;
