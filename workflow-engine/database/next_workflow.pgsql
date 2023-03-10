create or replace function workflow.next_workflow(
    executor_id bigint
) returns table(
    workflow_run_id bigint,
    is_valid boolean
)
language sql
as $$
select
    workflow_run_id,
    not exists(
        select 1
        from task.task_queue tq
        where
            tq.workflow_run_id = wr.workflow_run_id
            and tq.status not in (
                'Waiting'::task.task_status,
                'Complete'::task.task_status
            )
    ) is_valid
from workflow.workflow_runs wr
where
    status = 'Scheduled'::task.workflow_run_status
    and (executor_id is null or executor_id = $1)
limit 1
for update skip locked;
$$;

comment on function workflow.next_workflow IS $$
Get the next available workflow run for the given executor. Returns at most 1 row of a
workflow_run_id and a flag to indicate if the workflow run is valid or not. Invalid runs are reset
by the executor.

!NOTE! This function locks the record so this should be run within a transaction and once the
record is updated, immediately commit or rollback on error.

Arguments:
executor_id:
    ID of the executor to filter workflow runs (i.e. do not pick up workflow runs marked for
    another executor)
$$;
