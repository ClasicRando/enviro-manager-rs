create or replace function workflow_engine.create_task(
    name text,
    description text,
    task_service_id bigint,
    url text
) returns bigint
language sql
as $$
insert into workflow_engine.tasks(name,description,task_service_id,url)
values($1,$2,$3,$4)
returning task_id
$$;

comment on function workflow_engine.create_task IS $$
Create a new task, executable through the service referenced.

Arguments:
name:               Alias given to the new task
description:        Short description of the what the task does
task_service_id:    Id of the service that exeuctes the task
url:                Extension url to execute the task on the parent service
$$;
