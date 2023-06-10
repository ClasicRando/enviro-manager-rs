create or replace procedure workflow_run.restart_workflow_run(
    workflow_run_id bigint
)
security definer
language sql
as $$
update workflow_run.task_queue tq
set
    status = 'Waiting'::workflow_run.task_status,
    output = null,
    task_start = null,
    task_end = null
where tq.workflow_run_id = $1;

update workflow_run.workflow_runs wr
set
    status = 'Waiting'::workflow_run.workflow_run_status,
    executor_id = null
where wr.workflow_run_id = $1;
$$;

grant execute on procedure workflow_run.restart_workflow_run to we_web;

comment on procedure workflow_run.restart_workflow_run IS $$
Restart a given workflow run if possible. Updates all the tasks to a 'Waiting' state before setting
the workflow_run to 'Waiting'.

Arguments:
workflow_run_id:
    ID of the workflow run to be restarted
$$;
