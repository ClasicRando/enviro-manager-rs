create or replace procedure workflow_engine.aquire_next_task(
	p_workflow_run_id bigint,
	out workflow_run_id bigint,
	out task_order integer,
	out task_id bigint,
	out parameters jsonb,
	out url text
)
language plpgsql
as $$
begin
	start transaction;
	begin
		select workflow_run_id, task_order, task_id, parameters, url
		into $2, $3, $4, $5, $6
		from next_task($1)
		where task_order is not null;
		
		call start_task_run($1, $3);
	    commit;
	exception
		when no_data_found then
			commit;
			return;
		when others then
			rollback;
			raise;
	end;
end;
$$;

comment on procedure workflow_engine.aquire_next_task IS $$
Aquire the next task of a given input workflow_run_id. Attempts to fetch a next task and assign
that task to the specified workflow_run_id. If a next task is not available then it returns a null
object. If any error is raised then the transaction is rolled back and the exception is reraised.
$$;
