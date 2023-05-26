create or replace view executor.v_executors as
select
    re.executor_id, re.status, re.pid, re.username, re.application_name, re.client_addr,
    re.client_port, re.exec_start,
    exists (
        select 1
        from pg_stat_activity sa
        where sa.pid = re.pid
    ) as session_active,
    (
        select count(wr.workflow_run_id)
        from workflow.workflow_runs wr
        where wr.executor_id = re.executor_id
    ) as wr_count
from executor.executors re;

grant select on view executor.v_executors to we_web;

comment on view executor.v_executors IS $$
Utility view, showing all executors. Includes all base details of an executor, as well as a flag
indicating if the executor session is still active and the number of workflow runs the executor
owns.
$$;
