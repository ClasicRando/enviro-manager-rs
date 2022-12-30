create or replace view workflow_engine.v_executors as
select re.executor_id, re.pid, re.username, re.application_name,
       re.client_addr, re.client_port, re.exec_start,
       exists (
        select 1
        from   pg_stat_activity sa
        where  sa.pid = re.pid
       ) as session_active,
       (
        select count(wr.workflow_run_id)
        from   workflow_engine.workflow_runs wr
        where  wr.executor_id = re.executor_id
       ) as wr_count
from   workflow_engine.registered_we_executors re
where  re.status = 'Active'::workflow_engine.executor_status;

comment on view workflow_engine.v_executors IS $$
Utility view, showing active status executors. Includes all base details of an executor, as well as
a flag indicating if the executor session is still active and the number of workflow runs the
executor owns.
$$;
