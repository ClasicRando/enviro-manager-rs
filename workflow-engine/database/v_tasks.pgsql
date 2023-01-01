create or replace view workflow_engine.v_tasks as
select t.task_id, t.name, t.description, rtrim(ts.base_url,'/')||'/'||ltrim(t.url,'/') url,
       ts.name task_service_name
from   workflow_engine.tasks t
join   workflow_engine.task_services ts on t.task_service_id = ts.service_id;

comment on view workflow_engine.v_tasks IS $$
Utility view, showing tasks with the full url using the base url from the task service.
$$;
