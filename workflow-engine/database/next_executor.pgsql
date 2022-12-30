create function workflow_engine.next_executor()
returns bigint
language sql
stable
as $$
select executor_id
from   workflow_engine.executors
where  session_active
order by wr_count
limit 1;
$$;

comment on function workflow_engine.next_workflow IS $$
Get the next available executor to pick up a workflow run. Ensures the executor's session is active
and give priority to the executor with the least number of workflow runs. 

!NOTE! This function locks the record so this should be run within a transaction and once the
record is updated, immediately commit or rollback on error.

Arguments:
executor_id:    ID of the executor to filter workflow runs (i.e. do not pick up workflow runs
                marked for another executor)
$$;
