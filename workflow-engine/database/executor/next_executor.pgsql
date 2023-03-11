create or replace function executor.next_executor()
returns bigint
language sql
stable
as $$
select executor_id
from executor.v_executors
where session_active
order by wr_count
limit 1;
$$;

comment on function executor.next_executor IS $$
Get the next available executor to pick up a workflow run. Ensures the executor's session is active
and give priority to the executor with the least number of workflow runs.

Arguments:
executor_id:    ID of the executor to filter workflow runs (i.e. do not pick up workflow runs
                marked for another executor)
$$;
