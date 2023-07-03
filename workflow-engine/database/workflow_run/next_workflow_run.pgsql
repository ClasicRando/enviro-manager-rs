create or replace function workflow_run.next_workflow_run(
    executor_id bigint
)
returns table (
    workflow_run_id bigint,
    is_valid boolean
)
returns null on null input
security definer
language sql
as $$
select
    workflow_run_id,
    not exists(
        select 1
        from workflow_run.task_queue tq
        where
            tq.workflow_run_id = wr.workflow_run_id
            and tq.status not in (
                'Waiting'::workflow_run.task_status,
                'Complete'::workflow_run.task_status
            )
    ) is_valid
from workflow_run.workflow_runs wr
where
    status = 'Scheduled'::workflow_run.workflow_run_status
    and (executor_id is null or executor_id = $1)
limit 1
for update skip locked;
$$;

grant execute on function workflow_run.next_workflow_run to we_web;

comment on function workflow_run.next_workflow_run IS $$
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
