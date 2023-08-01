create or replace function workflow_run.next_task(
    in_workflow_run_id bigint,
    out workflow_run_id bigint,
    out task_order integer,
    out task_id bigint,
    out status workflow_run.task_status,
    out parameters jsonb,
    out url text
) returns record
security definer
language sql
volatile
as $$
select tq.workflow_run_id, tq.task_order, tq.task_id, tq.status, tq.parameters, t.url
from (
    select tq1.workflow_run_id, tq1.task_order, tq1.task_id, tq1.status, tq1.parameters
    from workflow_run.task_queue tq1
    where
        tq1.workflow_run_id = $1
        and not exists(
            select 1
            from workflow_run.task_queue tq2
            where
                tq1.workflow_run_id = tq2.workflow_run_id
                and tq2.status in (
                    'Running'::workflow_run.task_status,
                    'Paused'::workflow_run.task_status,
                    'Failed'::workflow_run.task_status,
                    'Rule Broken'::workflow_run.task_status
                )
        )
        and tq1.status = 'Waiting'::workflow_run.task_status
    order by tq1.task_order
    limit 1
    for update
) tq
join workflow.v_tasks t
on tq.task_id = t.task_id;
$$;

grant execute on function workflow_run.next_task to we_web;

comment on function workflow_run.next_task IS $$
Get the next available task for the given workflow_run_id. Returns at most 1 row of a row
containing data about the executable workflow_run.

!NOTE! This function locks the record so this should be run within a transaction and once the
record is updated, immediately commit or rollback on error.

Arguments:
workflow_run_id:
    ID of the workflow run to check for the next task
$$;
