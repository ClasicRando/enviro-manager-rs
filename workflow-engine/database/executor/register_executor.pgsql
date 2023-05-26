create or replace function executor.register_executor()
returns bigint
security definer
language sql
as $$
insert into executor.executors(pid,username,application_name,client_addr,client_port)
select a.pid, a.usename, a.application_name, a.client_addr, a.client_port
from pg_stat_activity a
where a.pid = pg_backend_pid()
returning executor_id;
$$;

revoke all on function executor.register_executor from public;
grant execute on function executor.register_executor to we_web;

comment on function executor.register_executor IS $$
Register a new workflow engine executor. Uses pg_stat_activity to populate details and returns
the new executor id generated.
$$;
