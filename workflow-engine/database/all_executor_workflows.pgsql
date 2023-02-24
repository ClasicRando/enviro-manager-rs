create or replace function workflow_engine.all_executor_workflows(
    executor_id bigint
) returns table(
    workflow_run_id bigint,
    status workflow_engine.workflow_run_status,
    is_valid boolean
)
language sql
stable
as $$
select
    wr.workflow_run_id, wr.status,
    not exists(
        select 1
        from workflow_engine.task_queue tq
        where
            tq.workflow_run_id = wr.workflow_run_id
            and q.status not in (
                'Waiting'::workflow_engine.task_status,
                'Complete'::workflow_engine.task_status
            )
    ) is_valid
from workflow_engine.workflow_runs wr
where wr.executor_id = $1;
$$;

comment on function workflow_engine.all_executor_workflows IS $$
Get all workflows linked to the given executor_id. Include current status and if the workflow is
valid (i.e. at least 1 of the tasks of a workflow run have a status that is not 'Waiting' or
'Complete').

Arguments:
executor_id:
    ID of the executor to collect workflow runs
$$;
