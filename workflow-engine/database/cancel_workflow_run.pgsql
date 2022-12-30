create procedure workflow_engine.cancel_workflow_run(
    workflow_run_id bigint
)
language sql
as $$
update workflow_engine.workflow_runs
set    status = 'Canceled'::workflow_engine.workflow_run_status,
       executor_id = null
where  workflow_run_id = $1
and    status = 'Running'::workflow_engine.workflow_run_status;

update workflow_engine.task_queue
set    status = 'Canceled'::workflow_engine.task_status,
       task_end = now() at time zone 'UTC',
       output = 'Workflow run canceled'
where  workflow_run_id = $1
and    status = 'Running'::workflow_engine.task_status;
$$;

comment on function workflow_engine.next_workflow IS $$
Cancel workflow run by setting workflow run status and updating any running tasks to the 'Canceled'
status with an appropriate output message.

Arguments:
workflow_run_id:    ID of the workflow to cancel
$$;
