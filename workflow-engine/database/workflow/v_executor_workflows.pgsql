create or replace view workflow.v_executor_workflows as
select
    wr.workflow_run_id, wr.status,
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
from workflow.workflow_runs wr;

revoke all on workflow.v_executor_workflows from public;
grant select on workflow.v_executor_workflows to we_web;

comment on view executor.all_executor_workflows IS $$
Get all workflows linked to an executor. Include current status and if the workflow is valid (i.e.
at least 1 of the tasks of a workflow run have a status that is not 'Waiting' or 'Complete').
$$;
