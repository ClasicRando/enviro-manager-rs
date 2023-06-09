create or replace procedure workflow_run.initialize_workflow_run(
    workflow_id bigint,
    out workflow_run_id bigint
)
language plpgsql
security definer
as $$
declare
    v_workflow_run_id bigint;
begin
    insert into workflow_run.workflow_runs(workflow_id)
    values($1)
    returning workflow_run_id into v_workflow_run_id;

    execute format(
        'create table "workflow_run".%I partition of workflow_run.task_queue for values in (%L)',
        'task_queue_'||v_workflow_run_id,
        v_workflow_run_id
    );

    insert into workflow_run.task_queue(workflow_run_id,task_order,task_id,parameters)
    select v_workflow_run_id, wt.task_order, wt.task_id, wt.parameters
    from workflow.workflow_tasks wt
    join workflow.tasks t on wt.task_id = t.task_id
    where wt.workflow_id = $1;

    return v_workflow_run_id;
end;
$$;

grant execute on procedure workflow_run.initialize_workflow_run to we_web;

comment on procedure workflow_run.initialize_workflow_run IS $$
Create a new workflow run entry and child tasks in task_queue using the workflow_id provided as a
template.

Arguments:
workflow_id:
    ID of the workflow that is used as a template to build the workflow run
$$;
