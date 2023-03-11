create or replace procedure task.update_task(
    task_id bigint,
    name text,
    description text,
    task_service_id bigint,
    url text
)
language sql
as $$
update task.tasks t
set
    name = $2,
    description = $3,
    task_service_id = $4,
    url = $5
where t.task_id = $1;
$$;

comment on procedure task.update_task IS $$
Update a task with new details. Always requires a full update.

Arguments:
task_id:
    Id of the task to modify
name:
    Alias given to the new task
description:
    Short description of the what the task does
task_service_id:
    Id of the service that executes the task
url:
    Extension url to execute the task on the parent service
$$;
