create or replace procedure workflow.fail_task_run(
    workflow_run_id bigint,
    task_order integer,
    message text
)
language plpgsql
as $$
begin
    if $3 is null or not data_check.check_not_blank_or_empty($3) then
        raise exception 'Message parameter must be non-null and not empty';
    end if;

    update task.task_queue tq
    set
        status = 'Failed'::task.task_status,
        output = $3,
        task_end = now() at time zone 'UTC'
    where
        tq.workflow_run_id = $1
        and tq.task_order = $2
        and tq.status = 'Running'::task.task_status;
end;
$$;

comment on procedure workflow.fail_task_run IS $$
Set the task record as 'Failed' with a required message to explain the failure

Arguments:
workflow_run_id:
    ID of the workflow run that owns the task
task_order:
    Task order within the workflow run to be run
output:
    Message output from the task run, must be non-null and not empty
$$;
