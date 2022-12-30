create or replace function workflow_engine.register_we_executor()
returns bigint
language sql
as $$
insert into workflow_engine.registered_we_executors(pid,username,application_name,client_addr,client_port)
select pid, usename, application_name, client_addr, client_port
from   pg_stat_activity
where  pid = pg_backend_pid()
returning executor_id;
$$;

comment on function workflow_engine.register_we_executor IS $$
Register a new workflow engine executor. Uses pg_stat_activity to populate details and returns
the new executor id generated.
$$;
