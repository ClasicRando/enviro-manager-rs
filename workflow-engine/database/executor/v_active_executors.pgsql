create or replace view executor.v_active_executors as
select
    re.executor_id, re.pid, re.username, re.application_name, re.client_addr, re.client_port,
    re.exec_start, re.session_active, re.wr_count
from executor.v_executors re
where exists (
    select 1
    from executor.executors e
    where
        e.executor_id = re.executor_id
        and e.status = 'Active'::executor.executor_status
);

comment on view executor.v_active_executors IS $$
Utility view, showing all active status executors. Includes all details found in v_executors
$$;
