create or replace function workflow_run.executor_workflows (
    executor_id bigint
)
returns table (
    workflow_run_id bigint,
    status workflow_run.workflow_run_status,
    is_valid boolean
)
language sql
security definer
as $$
select
    wr.workflow_run_id, wr.status,
    not exists (
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
where wr.executor_id = $1
$$;

grant execute on function workflow_run.executor_workflows to we_web;

comment on function workflow_run.executor_workflows IS $$
Get all workflows linked to an executor. Include current status and if the workflow is valid (i.e.
at least 1 of the tasks of a workflow run have a status that is not 'Waiting' or 'Complete').
$$;
