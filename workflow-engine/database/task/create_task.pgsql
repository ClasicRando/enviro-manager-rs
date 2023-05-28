create or replace function task.create_task(
    in_name text,
    in_description text,
    in_task_service_id bigint,
    in_url text
) returns bigint
security definer
language sql
as $$
insert into task.tasks as t (name,description,task_service_id,url)
values($1,$2,$3,$4)
returning t.task_id
$$;

grant execute on function task.create_task to we_web;

comment on function task.create_task IS $$
Create a new task, executable through the service referenced.

Arguments:
name:
	Alias given to the new task
description:
	Short description of the what the task does
task_service_id:
	Id of the service that executes the task
url:
	Extension url to execute the task on the parent service
$$;
