create or replace view task.v_tasks as
select
    t.task_id, t.name, t.description, rtrim(ts.base_url,'\')||'\'||ltrim(t.url,'\') url,
    ts.name task_service_name
from task.tasks t
join task.task_services ts on t.task_service_id = ts.service_id;

grant select on task.v_tasks to we_web;

comment on view task.v_tasks IS $$
Utility view, showing tasks with the full url using the base url from the task service.
$$;
