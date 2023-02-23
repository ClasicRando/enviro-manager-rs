create or replace function workflow_engine.create_task(
    p_name text,
    p_description text,
    p_task_service_id bigint,
    p_url text,
	out task_id bigint,
	out name text,
	out description text,
	out url text,
	out task_service_name text
) returns record
language plpgsql
as $$
declare
    v_task_id bigint;
begin
	insert into workflow_engine.tasks as t (name,description,task_service_id,url)
	values($1,$2,$3,$4)
	returning t.task_id into v_task_id;

	select v.task_id, v.name, v.description, v.url, v.task_service_name
	into $5, $6, $7, $8, $9
	from v_tasks v
	where v.task_id = v_task_id;

	return;
end;
$$;

comment on function workflow_engine.create_task IS $$
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
