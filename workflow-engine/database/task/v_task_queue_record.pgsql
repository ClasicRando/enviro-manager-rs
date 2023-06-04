create or replace view task.v_task_queue_record as
    select tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, t.url
    from task.task_queue tq
    join task.v_tasks t
    on t.task_id = tq.task_id;

grant select on task.v_task_queue_record to we_web;

comment on view task.v_task_queue_record IS $$
Utility view, allows accessing task_queue as read-only.
$$;
